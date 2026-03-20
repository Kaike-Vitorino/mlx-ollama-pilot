# Changelog por Usuario

Este arquivo registra contribuicoes por usuario, organizadas por sprint.

## Padrao para proximas sprints

Use sempre esta estrutura para facilitar continuidade:

- `Sprint N` (periodo)
  - `Contexto da sprint`
  - `Resumo de autoria` (quantidade de commits por pessoa)
  - `Detalhamento por usuario`
    - Commit(s)
    - Escopo tecnico
    - Resultado entregue

---

## Sprint 1 (Prototipo Inicial)

### Contexto da sprint

Objetivo: consolidar o prototipo inicial do MLX-Pilot com base local de inferencia, daemon HTTP, catalogo remoto, UI desktop e operacao via scripts.

### Resumo de autoria (Sprint 1)

- PETROMYZONMONSTER: 2 commits
- MarcellinhoHM: 2 commits
- gabriellima-4: 3 commits
- RamLi06: 3 commits
- GabrielSalustiano: 2 commits
- Kaike-Vitorino: commits restantes

### Detalhamento por usuario (Sprint 1)

#### PETROMYZONMONSTER

- Commit `8191085` - `chore: add base ignore rules and project overview`
  - Escopo tecnico: estrutura inicial de repositorio (`.gitignore`) e documentacao-base (`README.md`).
  - Resultado entregue: baseline do projeto definida para desenvolvimento e onboarding.

- Commit `d72cbe2` - `feat(tauri): add desktop shell bootstrap and capabilities`
  - Escopo tecnico: bootstrap do shell Tauri e capacidades iniciais da aplicacao desktop.
  - Resultado entregue: fundacao de execucao desktop para o cliente do prototipo.

#### MarcellinhoHM

- Commit `4e659fd` - `chore(workspace): add root cargo workspace manifests`
  - Escopo tecnico: manifests de workspace Cargo na raiz (`Cargo.toml`/`Cargo.lock`).
  - Resultado entregue: organizacao da monorepo Rust com build e dependencia centralizados.

- Commit `1ab8b23` - `feat(catalog): add remote model search and download jobs`
  - Escopo tecnico: modulo de catalogo remoto com busca e fila de jobs de download.
  - Resultado entregue: fluxo funcional para descoberta e obtencao de modelos.

#### gabriellima-4

- Commit `f9c4d83` - `feat(core): define chat domain types and provider contract`
  - Escopo tecnico: tipos de dominio de chat e contrato de provider em `crates/core`.
  - Resultado entregue: base de abstracao para provedores e pipeline de inferencia.

- Commit `e981268` - `feat(openclaw): add runtime bridge and status integration`
  - Escopo tecnico: integracao de runtime OpenClaw e status no daemon.
  - Resultado entregue: acoplamento funcional entre runtime externo e plano de controle.

- Commit `12cc722` - `chore(tauri): add generated schemas and app icon assets`
  - Escopo tecnico: schemas gerados e assets do aplicativo Tauri.
  - Resultado entregue: artefatos de configuracao/empacotamento desktop estabilizados.

#### RamLi06

- Commit `87d5297` - `feat(chat): add streaming runtime and metrics parsing`
  - Escopo tecnico: runtime de streaming de chat e parsing de metricas.
  - Resultado entregue: respostas incrementais com observabilidade tecnica de execucao.

- Commit `9e4c5bf` - `feat(desktop-ui): implement chat and discover interactions`
  - Escopo tecnico: interacoes de Chat e Discover na UI (`app.js`).
  - Resultado entregue: fluxo principal do usuario funcional no frontend desktop.

- Commit `8c7e10d` - `docs: add commit-level and user-level changelogs`
  - Escopo tecnico: documentacao de rastreabilidade por commit e por autor.
  - Resultado entregue: trilha historica inicial para auditoria da sprint.

#### GabrielSalustiano

- Commit `11df31e` - `chore(provider-mlx): add crate manifest`
  - Escopo tecnico: manifesto da crate do provider MLX.
  - Resultado entregue: estrutura formal do provider adicionada ao workspace.

- Commit `a794d7a` - `chore(scripts): add desktop run and daemon stop helpers`
  - Escopo tecnico: scripts operacionais para iniciar desktop e parar daemon.
  - Resultado entregue: rotina de execucao local simplificada para time e demos.

#### Kaike-Vitorino

- Commits `6529e02`, `02ffd14`, `17479ce`, `9670c73`, `0457863`, `c6fc5a7`.
  - Escopo tecnico: implementacao do provider MLX, dependencias/configuracao do daemon, wiring de rotas HTTP, base da UI desktop e manutencao dos changelogs apos redistribuicao de autoria.
  - Resultado entregue: integracao ponta-a-ponta do prototipo inicial e consolidacao da documentacao da sprint.
