<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface ProjectResult {
  name: string;
  status: string;
  added: number;
  modified: number;
  deleted: number;
  archive_path: string | null;
  error: string | null;
}

interface ArchiveAllResult {
  total: number;
  archived: number;
  skipped: number;
  errors: number;
  projects: ProjectResult[];
}

interface AppSettings {
  source_dir: string;
  archive_dir: string;
  exclude_patterns: string[];
}

const sourceDir = ref("");
const archiveDir = ref("");
const excludePatterns = ref("");
const running = ref(false);
const results = ref<ProjectResult[]>([]);
const summary = ref("");
const settingsLoaded = ref(false);
const showGuide = ref(false);

onMounted(async () => {
  try {
    const settings = await invoke<AppSettings>("load_settings");
    sourceDir.value = settings.source_dir;
    archiveDir.value = settings.archive_dir;
    excludePatterns.value = settings.exclude_patterns.join(", ");
    // 첫 실행: 경로가 비어있으면 가이드 표시
    if (!settings.source_dir && !settings.archive_dir) {
      showGuide.value = true;
    }
  } catch {
    showGuide.value = true;
  }
  settingsLoaded.value = true;
});

let saveTimer: ReturnType<typeof setTimeout> | null = null;
function autoSave() {
  if (!settingsLoaded.value) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      await invoke("save_settings", {
        settings: {
          source_dir: sourceDir.value,
          archive_dir: archiveDir.value,
          exclude_patterns: excludePatterns.value
            .split(",")
            .map((s) => s.trim())
            .filter((s) => s),
        },
      });
    } catch (e) {
      console.error("설정 저장 실패:", e);
    }
  }, 500);
}
watch([sourceDir, archiveDir, excludePatterns], autoSave);

async function browseFolder(target: "source" | "archive") {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    if (target === "source") {
      sourceDir.value = selected as string;
    } else {
      archiveDir.value = selected as string;
    }
    showGuide.value = false;
  }
}

function friendlyError(msg: string): string {
  if (msg.includes("소스 디렉토리가 존재하지 않습니다"))
    return "소스 디렉토리를 찾을 수 없습니다. 경로를 확인해 주세요.";
  if (msg.includes("다른 프로세스가 실행 중"))
    return "이미 다른 아카이브 작업이 실행 중입니다.";
  if (msg.includes("권한"))
    return "폴더 접근 권한이 없습니다. 관리자 권한으로 실행해 주세요.";
  if (msg.includes("IO 오류"))
    return "파일 읽기/쓰기 오류가 발생했습니다.";
  return msg;
}

async function runArchive(dryRun: boolean) {
  if (!sourceDir.value) {
    summary.value = "소스 디렉토리를 먼저 설정해 주세요.";
    return;
  }
  if (!archiveDir.value) {
    summary.value = "백업 디렉토리를 먼저 설정해 주세요.";
    return;
  }

  running.value = true;
  results.value = [];
  summary.value = "";

  try {
    const command = dryRun ? "archive_all_dry_run" : "archive_all";
    const res = await invoke<ArchiveAllResult>(command, {
      request: {
        source_dir: sourceDir.value,
        archive_dir: archiveDir.value,
        exclude_patterns: excludePatterns.value
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s),
      },
    });
    results.value = res.projects;
    summary.value = `전체: ${res.total}개 | 아카이브: ${res.archived}개 | 스킵: ${res.skipped}개 | 오류: ${res.errors}개`;
  } catch (e: any) {
    summary.value = friendlyError(String(e));
  } finally {
    running.value = false;
  }
}

const statusLabel: Record<string, string> = {
  archived: "완료",
  skipped: "변경없음",
  error: "오류",
  dry_run: "미리보기",
};
</script>

<template>
  <div class="container">
    <h1>📦 Arkiv</h1>

    <!-- 첫 실행 가이드 -->
    <div v-if="showGuide" class="guide">
      <p>처음 사용하시나요? 아래 단계를 따라 설정해 주세요.</p>
      <ol>
        <li>프로젝트들이 모여있는 <b>소스 디렉토리</b>를 선택하세요</li>
        <li>ZIP 파일을 저장할 <b>백업 디렉토리</b>를 선택하세요</li>
        <li><b>아카이브 실행</b> 버튼을 누르면 변경된 폴더만 ZIP으로 백업됩니다</li>
      </ol>
    </div>

    <div class="form-group">
      <label>소스 디렉토리</label>
      <div class="input-row">
        <input v-model="sourceDir" type="text" placeholder="프로젝트들이 있는 폴더 경로" />
        <button class="browse" @click="browseFolder('source')">찾기</button>
      </div>
    </div>

    <div class="form-group">
      <label>백업 디렉토리</label>
      <div class="input-row">
        <input v-model="archiveDir" type="text" placeholder="ZIP 파일을 저장할 폴더 경로" />
        <button class="browse" @click="browseFolder('archive')">찾기</button>
      </div>
    </div>

    <div class="form-group">
      <label>제외 패턴 (쉼표 구분)</label>
      <input v-model="excludePatterns" type="text" placeholder=".git, node_modules, *.tmp" />
    </div>

    <div class="actions">
      <button @click="runArchive(true)" :disabled="running">미리보기</button>
      <button @click="runArchive(false)" :disabled="running" class="primary">
        아카이브 실행
      </button>
    </div>

    <div v-if="running" class="status">⏳ 처리 중...</div>

    <div v-if="summary" class="summary" :class="{ error: summary.includes('오류') || summary.includes('설정') }">
      {{ summary }}
    </div>

    <div v-if="results.length" class="results">
      <div v-for="r in results" :key="r.name" class="result-item" :class="r.status">
        <div class="result-header">
          <span class="name">{{ r.name }}</span>
          <span class="badge" :class="r.status">{{ statusLabel[r.status] || r.status }}</span>
        </div>
        <div v-if="r.added || r.modified || r.deleted" class="changes">
          <span v-if="r.added" class="added">+{{ r.added }} 추가</span>
          <span v-if="r.modified" class="modified">~{{ r.modified }} 수정</span>
          <span v-if="r.deleted" class="deleted">-{{ r.deleted }} 삭제</span>
        </div>
        <div v-if="r.archive_path" class="archive-path">📁 {{ r.archive_path }}</div>
        <div v-if="r.error" class="error-msg">⚠️ {{ friendlyError(r.error) }}</div>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  --bg: #1a1a2e;
  --surface: #16213e;
  --accent: #e94560;
  --text: #eee;
  --text-dim: #999;
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: var(--bg); color: var(--text); font-family: "Segoe UI", sans-serif; }
.container { max-width: 860px; margin: 0 auto; padding: 24px; }
h1 { margin-bottom: 20px; font-size: 26px; }

.guide {
  background: #1b3a5c; border: 1px solid #2a5a8c; border-radius: 8px;
  padding: 16px; margin-bottom: 20px; font-size: 14px; line-height: 1.8;
}
.guide ol { padding-left: 20px; }

.form-group { margin-bottom: 14px; }
label { display: block; margin-bottom: 4px; color: var(--text-dim); font-size: 13px; }
.input-row { display: flex; gap: 8px; }
.input-row input { flex: 1; }
input {
  width: 100%; padding: 10px 12px; background: var(--surface);
  border: 1px solid #333; border-radius: 6px; color: var(--text); font-size: 14px;
}
input:focus { outline: none; border-color: var(--accent); }

.browse {
  padding: 10px 16px; border: 1px solid #444; border-radius: 6px;
  background: var(--surface); color: var(--text); cursor: pointer; font-size: 13px;
  white-space: nowrap;
}
.browse:hover { border-color: var(--accent); }

.actions { display: flex; gap: 12px; margin: 20px 0; }
button {
  padding: 10px 24px; border: 1px solid #444; border-radius: 6px;
  background: var(--surface); color: var(--text); cursor: pointer; font-size: 14px;
}
button:hover { border-color: var(--accent); }
button:disabled { opacity: 0.5; cursor: not-allowed; }
button.primary { background: var(--accent); border-color: var(--accent); }

.status { color: var(--accent); margin: 12px 0; font-size: 14px; }
.summary {
  padding: 12px; background: var(--surface); border-radius: 6px;
  margin-bottom: 16px; font-size: 14px;
}
.summary.error { border: 1px solid var(--accent); }

.results { display: flex; flex-direction: column; gap: 8px; }
.result-item { padding: 12px; background: var(--surface); border-radius: 6px; border-left: 3px solid #444; }
.result-item.archived { border-left-color: #4caf50; }
.result-item.skipped { border-left-color: #666; }
.result-item.error { border-left-color: var(--accent); }
.result-item.dry_run { border-left-color: #ff9800; }
.result-header { display: flex; justify-content: space-between; align-items: center; }
.name { font-weight: 600; font-size: 14px; }
.badge { font-size: 11px; padding: 2px 8px; border-radius: 10px; background: #333; }
.badge.archived { background: #1b5e20; }
.badge.skipped { background: #333; }
.badge.error { background: #b71c1c; }
.badge.dry_run { background: #e65100; }
.changes { margin-top: 6px; font-size: 13px; display: flex; gap: 12px; }
.added { color: #4caf50; }
.modified { color: #ff9800; }
.deleted { color: var(--accent); }
.archive-path { margin-top: 4px; font-size: 12px; color: var(--text-dim); word-break: break-all; }
.error-msg { margin-top: 4px; font-size: 12px; color: var(--accent); }
</style>
