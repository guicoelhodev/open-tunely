# Busca de músicas

Tauri 2 + Svelte. Pesquisa o primeiro resultado do YouTube pelo nome da música
ou artista e permite ouvir o resultado antes de baixar. O download ainda não está implementado.

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
Ao clicar em **Ouvir**, a interface chama `preview_song` com o ID do resultado.
O Rust solicita ao yt-dlp uma URL HTTPS temporária de áudio, preferindo M4A,
e o player nativo oferece reprodução, pausa, volume e navegação. Uma nova busca
interrompe a reprodução e cancela a extração anterior, encerrando o grupo de
processos do yt-dlp no Linux. A preparação tem limite de 45 segundos e apenas
uma extração de prévia executa por vez. Se o áudio falhar ou o endereço expirar,
**Tentar novamente** solicita uma nova URL. Nenhum arquivo de áudio é salvo pelo app.
O ícone de download está desativado por enquanto.

Esta etapa não usa FFmpeg ou runtimes JavaScript do sistema. A extração sem Deno
pode ter formatos indisponíveis em alguns vídeos; o suporte a um Deno próprio
e a conversão com FFmpeg ficam para uma próxima etapa. A reprodução também
depende dos codecs disponíveis no WebView do Linux.

## Biblioteca local

A biblioteca lateral permite escolher uma pasta pelo diálogo nativo e lista
os arquivos `.mp3` e `.MP3` diretamente nela, ordenados pelo nome. Subpastas e
links simbólicos não são incluídos. O nome exibido vem do arquivo, sem ler tags ID3.
Os botões de play/pause e os controles do player reproduzem os arquivos locais;
iniciar uma música local interrompe a prévia do YouTube e vice-versa.
Cancelar a seleção mantém a biblioteca atual. A pasta é mantida apenas na sessão.

Os ícones usam `@iconify/svelte` com módulos locais da coleção `andrsrxn`,
sem depender da API do Iconify. O protocolo de assets do Tauri está habilitado
com escopo inicial vazio; o Rust libera os MP3 listados após a escolha da pasta.

## Verificação

```sh
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```
