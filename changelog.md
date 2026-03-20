# Changelog

Este arquivo descreve o que foi feito em cada commit da branch `mvp-functional-a6f0f23` apos a divisao em commits menores e redistribuicao de autoria.

## Commits

- `8191085` - `chore: add base ignore rules and project overview`
  - Adiciona regras iniciais de ignorar arquivos (`.gitignore`).
  - Adiciona documentacao inicial do projeto (`README.md`).

- `4e659fd` - `chore(workspace): add root cargo workspace manifests`
  - Cria os manifests raiz do workspace Rust (`Cargo.toml` e `Cargo.lock`).

- `f9c4d83` - `feat(core): define chat domain types and provider contract`
  - Implementa a base de dominio em `crates/core` (tipos de chat, contratos e trait de provider).

- `11df31e` - `chore(provider-mlx): add crate manifest`
  - Adiciona o manifesto da crate do provider MLX.

- `6529e02` - `feat(provider-mlx): implement local model listing and inference`
  - Implementa listagem de modelos locais e inferencia no provider MLX.

- `02ffd14` - `chore(daemon): add daemon crate dependencies`
  - Define dependencias e configuracao da crate `daemon`.

- `17479ce` - `feat(daemon): add environment-driven runtime configuration`
  - Adiciona configuracao por variaveis de ambiente para o daemon.

- `1ab8b23` - `feat(catalog): add remote model search and download jobs`
  - Implementa catalogo remoto com busca de modelos e gerenciamento de jobs de download.

- `87d5297` - `feat(chat): add streaming runtime and metrics parsing`
  - Adiciona fluxo de streaming de chat e parsing de metricas de execucao.

- `e981268` - `feat(openclaw): add runtime bridge and status integration`
  - Integra runtime/bridge do OpenClaw e endpoints de status no daemon.

- `9670c73` - `feat(daemon): wire http routes for chat, catalog and openclaw`
  - Conecta as rotas HTTP principais (chat, catalogo e openclaw) no `main` do daemon.

- `0457863` - `feat(desktop-ui): add base static shell and styling`
  - Cria base da UI desktop (estrutura HTML, estilos e README da UI).

- `9e4c5bf` - `feat(desktop-ui): implement chat and discover interactions`
  - Implementa interacoes de chat e descoberta de modelos na UI (`app.js`).

- `d72cbe2` - `feat(tauri): add desktop shell bootstrap and capabilities`
  - Configura shell Tauri, bootstrap, capabilities e arquivos de configuracao de execucao.

- `12cc722` - `chore(tauri): add generated schemas and app icon assets`
  - Adiciona schemas gerados do Tauri e assets de icone do app.

- `a794d7a` - `chore(scripts): add desktop run and daemon stop helpers`
  - Adiciona scripts utilitarios para subir desktop e parar daemon.

- `8c7e10d` - `docs: add commit-level and user-level changelogs`
  - Adiciona documentacao de changelog por commit e por usuario.

- `c6fc5a7` - `docs: refresh changelogs after authorship redistribution`
  - Atualiza os changelogs para refletir a redistribuicao de autoria.
