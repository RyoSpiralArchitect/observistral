# OBSTRAL

Un cockpit de code "double cerveau" (deux panneaux):

- **Coder**: agit (fichiers + commandes) avec des approbations
- **Observer**: critique + propose les prochaines actions (avec score)
- **Chat**: brainstorming / narration sans casser l'execution

Languages: [English](README.md) | [Japanese](README.ja.md) | [French](README.fr.md)

## Qu'est-ce que c'est ?

La plupart des outils LLM optimisent la conversation.

OBSTRAL optimise des **boucles d'execution controlees**:

- tension a deux agents (Coder vs Observer)
- scoring des proposals + phase gating (core/feature/polish)
- detection de boucle (critiques repetitives / commandes en echec repetitives)
- garde-fous (approbation d'edition, approbation de commande, isolation via tool_root)

## Demarrage (serveur Rust)

### UI (Web)

```powershell
.\scripts\run-ui.ps1
```

Ouvrir:

- `http://127.0.0.1:18080/`

### TUI

```powershell
.\scripts\run-tui.ps1
```

Note: les scripts utilisent un `CARGO_TARGET_DIR` isole pour que UI et TUI puissent coexister.

## Serveur Lite (Python)

Si vous ne pouvez pas executer le binaire Rust (par ex. WDAC bloque les nouveaux EXE), il y a un fallback Python:

```powershell
python .\scripts\serve_lite.py
```

## Concepts

### tool_root

OBSTRAL execute toutes les actions de l'agent sous un dossier "scratch" (`tool_root`).

Par defaut: `.tmp/<thread-id>` pour isoler chaque thread et eviter les depots git imbriques.

### Approbations

- **Edit approval**: `write_file` est mis en file d'attente (pending edits) et applique apres approbation.
- **Command approval**: `exec` peut etre gate de la meme maniere (optionnel).

## Providers

OBSTRAL parle des APIs "OpenAI-compatible" et supporte plusieurs providers via un trait `ChatProvider`.

Erreurs frequentes:

- `401 Unauthorized`: cle API manquante/incorrecte
- `429 Too Many Requests`: rate limit (backoff)
- `max_tokens` vs `max_completion_tokens`: differences selon le modele

## Securite (local-first)

OBSTRAL est concu pour `127.0.0.1`.

Si vous l'exposez sur un reseau, ajoutez une authentification et durcissez l'execution d'outils.

## Depannage

### "Failed to connect to github.com via 127.0.0.1"

Votre environnement force probablement un proxy mort (`HTTP_PROXY/HTTPS_PROXY/ALL_PROXY`).

Dans PowerShell:

```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

### `cargo run` echoue: "access denied" sur `obstral.exe`

Le binaire est encore en cours d'execution depuis le meme target dir.

Utilisez:

- `.\scripts\kill-obstral.ps1`
- ou `.\scripts\run-ui.ps1` / `.\scripts\run-tui.ps1`

## Licence

MIT

