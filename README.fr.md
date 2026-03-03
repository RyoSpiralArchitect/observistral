# OBSTRAL

Un cockpit de code "double cerveau" (deux panneaux):

- **Coder**: agit (fichiers + commandes) avec des approbations
- **Observer**: critique + propose les prochaines actions (avec score)
- **Chat**: brainstorming / narration sans casser l'execution

Languages: [English](README.md) | [Japanese](README.ja.md) | [French](README.fr.md)

![OBSTRAL UI](docs/ui.png)

## Points Forts (Ce Qui Est Unique)

OBSTRAL n'est pas juste un client de chat. C'est un **moteur de controle de developpement** pour du "agentic coding".

- **Execution d'abord**: outils (`write_file`, `exec`) + approbations (human-in-the-loop)
- **Tension a deux agents**: Coder agit, Observer audite (et casse les boucles)
- **Moteur de proposals**: blocs `--- proposals ---` structures, scores, phases, impact/cout
- **Detection de boucle**: critiques repetitives (warning + effet visuel), commandes en echec repetitives (gouverneur)
- **Sandbox**: `tool_root` isole par thread pour eviter les depots git imbriques
- **Windows-reel**: PowerShell natif + serveur Lite Python (WDAC)

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

## Fonctionnalites

### Onglet Chat — Barre de chips de persona

Cinq chips de persona se trouvent au-dessus du compositeur Chat. Changez a tout moment — independant des personas du Coder et de l'Observer (defaut: 😊 Enjoue):

| Chip | Style |
|---|---|
| 😊 Enjoue (cheerful) | Enthousiaste et encourageant |
| 🤔 Reflechi (thoughtful) | Verifie les premisses, repond avec soin |
| 🧙 Sensei | Guide par les questions, pas les reponses |
| 😏 Cynique (cynical) | Va droit a la verite qui derange |
| 🦆 Canard (duck) | Ne repond jamais — pose juste « Pourquoi ? » |

### Observer — Badge de sante `❤ N`

Quand l'Observer emet un bloc `--- health ---`, le score apparait dans la barre d'etat:

| Score | Couleur | Signification |
|---|---|---|
| ≥ 70 | Vert | Pret pour la production |
| 40–69 | Ambre | OK pour dev / demo |
| < 40 | Rouge | Action immediate requise |

### Observer — Cycle de vie du statut de proposition

Si l'Observer souleve la meme proposition a plusieurs reprises sans qu'elle soit traitee, le statut monte en grade:

| Statut | Signification | Bonus de score |
|---|---|---|
| `new` | Premiere apparition | ±0 |
| `[UNRESOLVED]` | Ignoree une fois | +10 |
| `[ESCALATED]` | Ignoree deux fois ou plus — forcee en tete | +20 au total |
| `addressed` | Traitee (affichee en cyan) | — |

### Champ `quote`

Obligatoire pour les propositions `warn` / `critical`. Affiche l'extrait incrimine en monospace cyan sur la carte:

```
❝ user_input = input()
```

---

## Concepts

### tool_root

OBSTRAL execute toutes les actions de l'agent sous un dossier "scratch" (`tool_root`).

Par defaut: `.tmp/<thread-id>` pour isoler chaque thread et eviter les depots git imbriques.

### Approbations

- **Edit approval**: `write_file` est mis en file d'attente (pending edits) et applique apres approbation.
- **Command approval**: `exec` est mis en file d'attente (pending commands) et approuve/rejete (le Coder reprend apres approbation).

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

### Push via SSH sur le port 443

Dans des reseaux verrouilles, SSH over 443 est souvent le plus fiable:

```powershell
.\scripts\push_ssh.ps1
```

### Push sans invite interactive (compatible WDAC)

Dans certains environnements, les invites git interactives cassent (par ex. `sh.exe` echoue avec Win32 error 5).

Si vous avez un token GitHub, vous pouvez push en non-interactif:

```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

### `cargo run` echoue: "access denied" sur `obstral.exe`

Le binaire est encore en cours d'execution depuis le meme target dir.

Utilisez:

- `.\scripts\kill-obstral.ps1`
- ou `.\scripts\run-ui.ps1` / `.\scripts\run-tui.ps1`

## Licence

MIT
