# Busca de músicas

Tauri 2 + Svelte. Pesquisa o primeiro resultado do YouTube pelo nome da música
ou artista e exibe o título. Esta etapa não baixa áudio.

## Desenvolvimento (Linux x64)

Com os pré-requisitos de desenvolvimento do Tauri instalados:

```sh
npm install
npm run setup:tools
npm run dev
```

`npm run dev` abre a janela desktop do Tauri e inicia o Vite automaticamente.
Faça as buscas nessa janela: abrir `localhost:1420` no navegador mostra apenas
a interface, sem acesso ao backend Rust. `npm run dev:web` inicia somente a
interface para ajustes visuais. `npm run tauri dev` também continua disponível.

O setup baixa o executável independente oficial do yt-dlp 2026.10.26,
verifica o SHA-256 fixado no script e salva em `src-tauri/binaries/yt-dlp`.
O binário não é versionado; o setup reaproveita a cópia se o checksum confere.
O aplicativo usa exclusivamente essa cópia, sem procurar yt-dlp no PATH.
O arquivo também está configurado como recurso para futuros builds locais.

A interface chama `search_song` no Rust. O backend executa
`ytsearch1:<consulta>` com `--flat-playlist --skip-download --dump-json`
e retorna `{ id, title }`. O processo roda fora da thread da interface.
Esta busca simplificada não precisa de FFmpeg ou Deno; eles serão acrescentados
na etapa de download/conversão.

## Verificação

```sh
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```
