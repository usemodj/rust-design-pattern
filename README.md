# Rust 디자인 패턴 한국어 안내서

Axum으로 만든 한국어 Markdown 문서 사이트입니다. 문서는 `content/ko/`에 있으며,
왼쪽 목차는 원문 책의 장·절 계층을 따라 관용구, 디자인 패턴의 하위 유형, 안티 패턴,
함수형 프로그래밍과 추가 자료를 단계적으로 탐색하도록 구성했습니다. 각 항목 옆의 화살표로
하위 목차를 접거나 펼칠 수 있으며, 현재 읽는 문서의 경로는 자동으로 펼쳐집니다.
상단 도구 모음에서는 목차 표시를 전환하고, 51개 문서의 제목과 본문을 검색하며,
어두운·밝은·세피아·오션·포레스트 테마를 선택할 수 있습니다. 테마 설정은 브라우저에 저장됩니다.
각 문서 하단에는 목차 순서를 따르는 이전/다음 페이지 이동 링크가 표시됩니다.

## 실행

프로젝트 루트의 `.env`에서 서버 호스트와 포트를 설정합니다. 새로 내려받았다면
`.env.example`을 `.env`로 복사하세요.

```powershell
Copy-Item .env.example .env
```

```dotenv
HOST=0.0.0.0
PORT=3000
```

`.env`가 없거나 항목이 빠지면 위 값이 기본값입니다. 기존 환경 변수가 `.env`보다
우선하며, 잘못된 포트나 빈 호스트는 오류로 처리합니다. `HOST=0.0.0.0`은 모든
IPv4 인터페이스에서 접속을 허용합니다. 같은 Wi-Fi의 스마트폰에서는
`http://<PC의 IPv4 주소>:3000`으로 접속하며, Windows 방화벽에서도 해당 포트를
허용해야 합니다. PC에서만 접속하려면 `HOST=127.0.0.1`로 설정하세요.
`.env`는 Git에서 제외되며, 설정을 변경한 후 서버를 다시 시작해야 합니다.

```powershell
cargo run
```

기본 설정에서는 브라우저에서 <http://127.0.0.1:3000>을 여세요. 상태 확인 주소는
<http://127.0.0.1:3000/health>입니다. 포트를 변경했다면 주소에도 변경된 포트를 사용하세요.

## 번역 범위

원문의 51개 문서에 대응하는 한국어 Markdown 페이지를 제공합니다. 설명은 한국어로
옮기고 Rust 코드와 식별자는 원문을 유지했습니다. 원문은 사이트 각 페이지에서 확인할 수 있습니다.
각 문서 하단의 GitHub 링크는 해당 문서의 영문 원문 파일로 연결됩니다.

## 출처와 라이선스

번역 및 요약의 기반이 된 [Rust Design Patterns](https://github.com/rust-unofficial/patterns)는
MPL-2.0으로 배포됩니다. 라이선스 전문은 [LICENSE](LICENSE)를 참고하세요.
한국어 번역 문서의 출처와 범위는 [NOTICE.md](NOTICE.md)에 기록되어 있습니다.

----

1. GitHub 저장소 만들기
GitHub 새 저장소 만들기에서 저장소 이름을 입력하고 공개 여부를 선택하세요. README, .gitignore, 라이선스는 추가하지 않고 빈 저장소로 만듭니다. 프로젝트에 이미 해당 파일들이 있습니다.

2. 프로젝트에서 Git 초기화

```sh
git init -b main
git add .
git status
git diff --cached
```

3. 커밋하고 GitHub에 올리기
아래 주소에서 사용자이름과 저장소이름을 실제 값으로 바꾸세요.

`
git commit -m "Initial commit"
git remote add origin https://github.com/사용자이름/저장소이름.git
git push -u origin main
```

 이후 변경 사항을 올릴 때는 다음을 실행하면 됩니다.

```sh
git add .
git status
git diff --cached
git commit -m "변경 내용"
git push
```
