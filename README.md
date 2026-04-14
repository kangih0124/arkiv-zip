# Arkiv

Arkiv는 여러 프로젝트 폴더를 한 번에 스캔하고, 변경된 폴더만 ZIP으로 아카이브하는 데스크톱 앱입니다.  
프런트엔드는 Vue 3, 데스크톱 런타임과 백엔드는 Tauri + Rust로 구성되어 있습니다.

## 주요 기능

- 소스 디렉토리 하위의 여러 프로젝트 폴더를 일괄 스캔
- 변경된 프로젝트만 ZIP 아카이브 생성
- 실행 전 미리보기 모드 제공
- 프로젝트별 `index.json` 기반 상태 추적
- 파일 추가, 수정, 삭제 개수 표시
- 제외 패턴 설정 지원
- 설정 자동 저장

## 현재 동작 방식

Arkiv는 사용자가 지정한 소스 디렉토리의 1단계 하위 폴더를 각각 하나의 프로젝트로 간주합니다.

예:

```text
D:\work\projects
  project-a
  project-b
  project-c
```

이 경우 `project-a`, `project-b`, `project-c` 각각이 독립적으로 스캔되고, 변경된 프로젝트만 ZIP으로 저장됩니다.

각 프로젝트 폴더 내부에는 `index.json`이 생성되며, 여기에는 다음 정보가 저장됩니다.

- 마지막 실행 시점
- 최신 파일 상태
- 변경 이력
- 생성된 아카이브 정보

## 사용자 흐름

1. 소스 디렉토리를 선택합니다.
2. ZIP 저장용 아카이브 디렉토리를 선택합니다.
3. 필요하면 제외 패턴을 입력합니다.
4. `미리보기`로 변경 예상 결과를 확인합니다.
5. `아카이브 실행`으로 실제 ZIP 파일을 생성합니다.

## 화면에서 제공하는 기능

- `소스 디렉토리` 선택
- `백업 디렉토리` 선택
- 제외 패턴 편집
- 미리보기 실행
- 실제 아카이브 실행
- 프로젝트별 처리 결과 확인

프로젝트별 결과에는 다음 정보가 표시됩니다.

- 상태: 완료 / 변경없음 / 오류 / 미리보기
- 추가 파일 수
- 수정 파일 수
- 삭제 파일 수
- 생성된 ZIP 경로

## 설정 저장 위치

앱 설정은 운영체제 표준 설정 폴더 아래에 저장됩니다.

- Windows: `%APPDATA%\arkiv\config.json`
- Linux/macOS: `~/.config/arkiv/config.json`

저장 내용:

- `source_dir`
- `archive_dir`
- `exclude_patterns`

## 프로젝트 메타데이터

현재 버전은 각 프로젝트 폴더 내부에 `index.json`을 생성해 상태와 이력을 관리합니다.

이 파일에는 아래 정보가 포함됩니다.

- 현재 파일 상태
- 변경 이력
- 생성된 아카이브 목록

자동으로 관리되는 내부 파일:

- `index.json`
- `.arkiv.lock`

이 두 파일은 자동으로 제외 패턴에 포함됩니다.

## 아카이브 산출물

현재 배포 버전에서 실제 생성되는 산출물은 ZIP입니다.

파일명 형식:

```text
<project-name>_YYYYMMDD_HHMMSS.zip
```

예:

```text
project-a_20260414_103015.zip
```

## 기술 스택

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust

## 개발 환경 실행

필수 준비:

- Node.js
- Rust
- Tauri 빌드 환경

개발 실행:

```bash
npm install
npm run tauri dev
```

프런트엔드만 확인:

```bash
npm run dev
```

## 배포 빌드

프런트엔드 빌드:

```bash
npm run build
```

Tauri 배포 빌드:

```bash
npm run tauri build
```

배포 산출물은 Tauri 기본 출력 경로에 생성됩니다.

일반적으로:

```text
src-tauri/target/release/bundle/
```

## 현재 범위와 제한 사항

- 현재 UI는 ZIP 아카이브 생성과 미리보기 중심입니다.
- Rust 백엔드에는 복원 및 검증 명령이 존재하지만, 현재 메인 UI에서는 노출되지 않습니다.
- 아카이브 형식은 현재 ZIP만 사용합니다.
- 프로젝트 메타데이터는 프로젝트 폴더 내부의 `index.json`에 저장됩니다.

## 배포 전 확인 권장 사항

- 소스 디렉토리와 아카이브 디렉토리가 서로 다른지 확인
- 프로젝트 폴더에 `index.json` 생성이 가능한지 확인
- 아카이브 디렉토리에 ZIP 쓰기 권한이 있는지 확인
- 제외 패턴이 실제 운영 환경에 맞는지 확인

## 문서

- 배포 가이드: [docs/deployment.md](D:/toy/work_projectzip/docs/deployment.md)
- 제품/기능 요구사항: [docs/requirements.md](D:/toy/work_projectzip/docs/requirements.md)
- 기술 설계 참고: [docs/design.md](D:/toy/work_projectzip/docs/design.md)
