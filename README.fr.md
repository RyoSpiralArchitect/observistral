# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

> **Une seule boite de dialogue ne suffit pas.**
> OBSTRAL donne a votre IA un deuxieme cerveau — et les fait se disputer.

Languages: [English](README.md) | [日本語](README.ja.md) | [Français](README.fr.md)

---

Tous les outils de code IA ont le meme probleme : le modele qui ecrit votre code est aussi celui qui le relit.

Ce n'est pas une revue de code. C'est un monologue de defense.

OBSTRAL resout ca en faisant tourner Coder et Observer dans des **contextes entierement separes**. L'Observer n'a vu aucune ligne de votre code en cours d'ecriture. Il ne connait que le resultat. C'est ce qui le rend honnete.

---

## Pourquoi OBSTRAL existe

La plupart des outils LLM optimisent la conversation.
OBSTRAL optimise des boucles d'execution controlees : roles separes, garde-fous d'approbation, et critique cumulative au lieu de repartir de zero a chaque tour.

Ce n'est pas un client de chat.
C'est un moteur de controle du processus de developpement.

---

## Trois roles. Trois contextes. Zero conflit.

| Role | Ce qu'il fait | Ce qu'il ne fait jamais |
|---|---|---|
| **Coder** | Agit — fichiers, commandes shell, boucle agentique (12 etapes max), 5 outils integres | Relire ou remettre en question son propre travail |
| **Observer** | Critique — score chaque proposition, escalade ce que vous ignorez | Toucher au code. Il lit seulement. |
| **Chat** | Reflechit avec vous — conception, canard en plastique, compromis | Interrompre la boucle d'execution |

Roles distincts. Modeles distincts si vous le souhaitez. Contextes toujours distincts.

---

## Ce qu'OBSTRAL sait avant que vous parliez

Quand vous definissez `tool_root`, OBSTRAL analyse automatiquement le projet :

```
[Project Context — auto-detected]
stack: Rust, React (no bundler)
git:   branch=main  modified=2  untracked=1
recent: "fix(observe): require all 4 blocks" · "feat(agent): error classifier"
tree:
  src/          12 files  (Rust source)
  web/           4 files  (JS/CSS)
  scripts/       8 files  (PowerShell)
key:  Cargo.toml · web/app.js · README.md
```

Ce contexte est injecte dans le message systeme du Coder **avant votre premier prompt**. Quand vous commencez a taper, le Coder connait deja le stack, la branche courante, les fichiers modifies et l'arborescence.

Dans le TUI, un badge s'affiche en temps reel dans l'entete : `▸ Rust · React · git:main`
Dans l'UI Web, le label du stack apparait sous le champ toolRoot dans les parametres.

**Detection du stack** — OBSTRAL cherche les fichiers manifestes :
- `Cargo.toml` → Rust
- `package.json` → Node / React / TypeScript (inspecte les deps)
- `pyproject.toml` / `requirements.txt` → Python
- `go.mod` → Go
- `pom.xml` → Java

L'analyse s'execute une fois par session, prend moins de 200 ms et ignore silencieusement ce qu'elle ne peut pas lire.

---

## Ce qui rend OBSTRAL different

### L'Observer n'a rien a defendre

Autres outils : meme modele ecrit le code → meme modele le relit → modele defend ses propres choix.

OBSTRAL : contexte vierge a chaque execution de l'Observer. Il ne sait pas ce qu'il *aurait* ecrit. Il juge uniquement ce qu'il voit.

Resultat : feedbacks plus tranchants, evaluation de risques honnete, pas de demi-mesures.

### Les propositions ne disparaissent pas

Quand l'Observer signale un probleme que vous ignorez, la proposition monte en grade :

```
new  →  [UNRESOLVED] +10pts  →  [ESCALATED] +20pts, epinglee en haut
```

L'Observer se souvient de ce qu'il a dit. Si vous ignorez un avertissement `critical` deux fois, il devient la carte la plus visible du tableau.

### Classification d'erreur, pas juste des codes de sortie

Quand une commande echoue, OBSTRAL ne donne pas au modele un brut `exit_code: 1` en esperant le mieux. Il classe l'erreur d'abord :

| Type d'erreur | Hint injecte |
|---|---|
| `ENVIRONMENT` | Corrigez l'environnement. Ne touchez pas au code source. |
| `SYNTAX` | Corrigez le fichier exact. Ne changez pas d'autre code. |
| `PATH` | Verifiez les chemins d'abord. Ne creez pas avant de confirmer. |
| `DEPENDENCY` | Installez le package d'abord. Puis reessayez. |
| `NETWORK` | Verifiez le service et les variables proxy. |
| `LOGIC` | Relisez la logique. Ne relancez pas juste pour relancer. |

Note PowerShell : `exit_code` peut etre `0` meme si des erreurs ont ete imprimees (erreurs non bloquantes).
OBSTRAL le signale comme `SUSPICIOUS_SUCCESS` et le traite comme un echec pour eviter les faux progres.

### Le Coder dispose de cinq outils

Le Coder n'est pas limite aux commandes shell. Il dispose de cinq outils dedies :

| Outil | Quand l'utiliser |
|---|---|
| `exec(command, cwd?)` | Build, tests, git, installation de packages — tout ce qui est shell |
| `read_file(path)` | Lire le contenu exact d'un fichier sans problemes de guillemets shell |
| `write_file(path, content)` | Creer ou ecraser un fichier de maniere atomique (repertoires parents auto-crees) |
| `patch_file(path, search, replace)` | Remplacer un extrait exact — echoue bruyamment en cas d'ambiguite |
| `apply_diff(path, diff)` | Appliquer un diff unifie `@@` (plusieurs hunks) — ideal quand `patch_file` est trop petit |

`write_file`, `patch_file` et `apply_diff` utilisent un schema fichier temporaire → renommage, donc un crash en cours d'ecriture ne laisse jamais de fichier corrompu.

`patch_file` exige que la chaine de recherche apparaisse **exactement une fois**. Zero occurrence → apercu du fichier pour auto-correction. Plusieurs occurrences → compte exact retourne en erreur. L'ambiguite est une erreur, pas une supposition.

**Marqueurs visuels dans le TUI** pour identifier l'outil utilise d'un coup d'oeil :
- `📄 READ` (bleu-vert) — fichier lu
- `✎ WRITE` (bleu) — fichier cree ou ecrase
- `⟳ PATCH` (magenta) — extrait remplace
- `✓` (vert) / `✗` (rouge) — succes / erreur

### Le Coder se remet en question

Avant chaque appel d'outil, le Coder remplit un bloc de 5 lignes :

```
<think>
goal:   ce qui doit reussir maintenant
risk:   le mode d'echec le plus probable
doubt:  une raison pour laquelle cette approche pourrait etre fausse   ← le champ inhabituel
next:   commande ou operation exacte
verify: comment confirmer que ca a fonctionne
</think>
```

Le champ `doubt:` force le modele a formuler un doute avant d'agir. ~50 tokens. Ca empeche le mode d'echec ou le modele est confiant et faux.

### Boucle a machine d'etats (Planning → Executing → Verifying → Recovery)

Beaucoup de "agent loops" ne sont qu'un timer. OBSTRAL route le Coder via une petite machine d'etats :

- `planning`  — reformuler l'objectif et choisir la prochaine etape concrete
- `executing` — executer des outils (fichiers/commandes)
- `verifying` — lancer des probes `goal_check` avant de declarer "done"
- `recovery`  — detection de blocage -> diagnostics + changement de strategie

Cela aide les runs longs a converger au lieu de tourner en rond.

OBSTRAL injecte aussi une memoire compacte `[Recent runs]` (commandes + `write_file` / `patch_file` / `apply_diff`) pour que le Coder n'oublie pas ce qu'il vient de faire et ne repete pas les memes actions.

### Verification des objectifs a l'arret (pas de faux "Done")

Quand le modele renvoie `finish_reason=stop` sans tool calls, OBSTRAL peut executer automatiquement des checks legers (repo init, tests, build) et injecter un message `[goal_check]` dans la boucle si quelque chose manque ou echoue.

### References @fichier : sautez le tour de lecture

Tapez `@chemin` n'importe ou dans votre message pour injecter le contenu du fichier comme contexte avant que votre prompt atteigne le Coder :

```
@src/main.rs que fait run_chat ?
@Cargo.toml @package.json montre-moi les versions de dependances cote a cote
corrige le bug dans @src/server.rs ligne 400
```

Le TUI affiche une notification pour chaque fichier injecte :
```
📎 injected: [src/main.rs] (276 lines, 8192 bytes)
```

L'UI Web affiche des chips dans le compositeur pendant la saisie :
```
📎 @src/main.rs   📎 @Cargo.toml
```

Le Coder voit le contenu du fichier immediatement — pas de tour `read_file` supplementaire. Avec un budget de 12 iterations, economiser un tour de lecture peut faire la difference entre succes et timeout.

### Phase gating : taire le bon bruit

Dites a l'Observer la phase dans laquelle vous etes (`core` / `feature` / `polish`). Les propositions qui ne correspondent pas sont automatiquement estompees. Les retouches CSS ne vous interrompent pas quand votre auth est cassee.

### Sante en un coup d'oeil

Chaque reponse de l'Observer se termine par un score :

```
--- health ---
score: 74  rationale: auth is solid, tests cover happy path only
```

❤ **74** → vert (zone production). Le badge se met a jour en direct.

### Points de controle de progression

Aux iterations 3, 6 et 9, le Coder s'arrete pour une auto-evaluation :

```
1. DONE: quelles etapes du plan sont verifiees completes (exit_code=0) ?
2. REMAINING: qu'est-ce qui reste ?
3. ON_TRACK: oui/non — si non, reevalue le plan avant la prochaine operation.
```

C'est la difference entre un agent qui tourne en rond et un qui sait quand il est perdu.

### Windows en premier (vraiment)

La plupart des outils de code IA sont concus sur Mac, testes sur Linux, et "devrait marcher" sur Windows.

OBSTRAL a ete construit sur Windows. Il gere :
- Binaires bloques par WDAC → serveur de secours Python Lite (stdlib pure)
- Traduction automatique de syntaxe PowerShell (bash → PS)
- L'enfer des proxies d'entreprise
- `sh.exe` Win32 error 5 sur les invites git interactives

### Registre de plugins

Etendez OBSTRAL sans le forker :

```js
registerObserverPlugin({ name: "mon-plugin", onProposal, onHealth, onPhase })
registerPhase("security-review", { label: "Revue securite", color: "#f97316" })
registerValidator(propositions => propositions.filter(p => p.score > 20))
```

Chargez votre plugin via `<script>` avant `app.js`. C'est tout.

---

## Le contrat de sortie de l'Observer

L'Observer n'ecrit pas librement. Il parle un format structure que l'UI transforme en cartes :

```
--- phase ---
core

--- proposals ---
title: Validation des entrees manquante
toCoder: Validez la longueur et le type avant de traiter l'entree utilisateur.
severity: critical
score: 88
phase: core
cost: low
impact: empeche le crash sur entree malformee
quote: user_input = input()

--- critical_path ---
Corrigez la validation des entrees avant d'ajouter de nouvelles fonctionnalites.

--- health ---
score: 41  rationale: la logique centrale fonctionne mais la surface d'injection est grande ouverte
```

Chaque champ est intentionnel. `quote` epingle la ligne exacte incriminee sur la carte. `cost` dit si la correction est facile avant de lire les details. `phase` controle la visibilite.

---

## Demarrage rapide

**UI Web (recommande)**
```powershell
.\scripts\run-ui.ps1
# → http://127.0.0.1:18080/
```

**TUI (terminal)**
```powershell
.\scripts\run-tui.ps1
```

**Coder headless (CLI)**
```powershell
# (optionnel) generer le template .obstral.md (stack + test_cmd)
obstral init -C .

# lancer l'agent de code dans votre projet
obstral agent "fix the failing test" -C . --vibe --session
# reprendre plus tard (sans prompt -> "continue" auto)
obstral agent -C . --vibe --session

# artefacts (trace JSONL + snapshot JSON final + graphe d'execution)
obstral agent "fix the failing test" -C . --vibe --trace-out .tmp/obstral_trace.jsonl --json-out .tmp/obstral_final.json --graph-out .tmp/obstral_graph.json
```

**Python Lite (WDAC / pas de binaire Rust)**
```powershell
python .\scripts\serve_lite.py
# → http://127.0.0.1:18080/
```

---

## Concepts cles

### tool_root

Chaque action de l'agent s'execute dans un repertoire de travail.

Par defaut :
- **UI Web** : `.tmp/<thread-id>` (isole par thread)
- **TUI** : `.tmp/tui_<epoch>` (isole par session)
- **CLI** : repertoire courant

Pour travailler sur votre projet reel, definissez `tool_root` sur le chemin du projet :
- **TUI** : option `-C .` / `--tool-root .`, ou commande slash `/root <chemin>` en cours de session
- **UI Web** : Parametres → champ toolRoot

Quand `tool_root` est defini, OBSTRAL l'analyse a la premiere utilisation pour construire le bloc de contexte projet (stack, git, arborescence). Les envois suivants dans la meme session sautent l'analyse.

La traversee de chemins est bloquee : les chemins avec des composantes `..` sont rejetes a chaque frontiere d'outil (jamais silencieusement).

### Langue

- **Langue de l'UI** : TUI `/lang ja|en|fr` (affecte aussi les prompts).
- **Langue de l'Observer (UI Web)** : `auto` (par defaut) suit la langue de votre dernier message utilisateur meme si l'UI est en anglais ; `ui` suit l'UI ; ou forcez `ja`/`en`/`fr`.

### Sessions (CLI)

`obstral agent` peut sauvegarder et reprendre la conversation complete (y compris les tool calls) avec `--session[=<chemin>]`.

- Chemin par defaut : `.tmp/obstral_session.json`
- Si `-C/--root` est defini, les chemins relatifs de sortie sont resolus sous `tool_root`
- Auto-sauvegarde pendant l'execution (apres les tool calls)
- Reprendre sans prompt : relancez `obstral agent -C . --session`
- Recommencer : ajoutez `--new-session` (ecrase le fichier)

Artefacts associes :
- `--trace-out <chemin>` / `--trace_out` : trace JSONL (tool calls, checkpoints, errors, done)
- `--json-out <chemin>` / `--json_out` : snapshot JSON final (messages + tool calls + tool results)
- `--graph-out <chemin>` / `--graph_out` : graphe d'execution JSON (noeuds + aretes) derive des messages finaux
- Si `-C/--root` est defini, les chemins relatifs sont resolus sous `tool_root`

Le JSON de session/trace peut contenir du code et des sorties d'outils; traitez-le comme sensible.

### Approbations

- **Edit approval** : les appels `write_file` sont mis en file d'attente comme pending edits. Vous approuvez ou rejetez chacun.
- **Command approval** : les appels `exec` peuvent etre gates de la meme maniere (optionnel). Le Coder attend votre decision puis reprend.

Aucun mode ne vous force a vous arreter — ils se mettent en file d'attente silencieusement.

### Providers

OBSTRAL parle les APIs OpenAI-compatibles. Il supporte aussi Mistral, Anthropic, Gemini et les modeles HF locaux via un trait `ChatProvider`.

Definissez un modele different par role : modele rapide pour les iterations du Coder, modele puissant pour l'analyse de l'Observer. Erreurs courantes : `401` (cle incorrecte), `429` (rate limit), mismatch `max_tokens` vs `max_completion_tokens`.

### Personas Chat

Cinq chips au-dessus du compositeur Chat — changez a tout moment, independant du Coder/Observer :

| Chip | Style |
|---|---|
| 😊 Enjoue (cheerful) | Enthousiaste et encourageant |
| 🤔 Reflechi (thoughtful) | Verifie les premisses, repond avec soin |
| 🧙 Sensei | Guide par les questions, pas les reponses |
| 😏 Cynique (cynical) | Va droit a la verite qui derange |
| 🦆 Canard (duck) | Ne repond jamais — pose juste « Pourquoi ? » |

### Chat = Compagnon (pas un agent)

Le Chat n'execute jamais d'outils. Il est la pour vous garder dans le flux pendant que le runtime (Coder/Observer) tourne.

Dans l'UI Web, deux aides optionnelles :
- **Joindre snapshot runtime** : injecte un petit resume en lecture seule (cwd, dernier extrait d'erreur, approbations en attente, taches ouvertes) pour demander "que se passe-t-il ?" sans quitter l'onglet Chat.
- **Tâches auto** : un TaskRouter en coulisses transforme la discussion en taches concretes pour Coder/Observer (visibles dans **Tâches**). Vous choisissez toujours quoi envoyer.

### Commandes slash (TUI)

| Commande | Effet |
|---|---|
| `/model <nom>` | Changer de modele en cours de session |
| `/persona <cle>` | Changer le persona du Coder |
| `/temp <0.0–1.0>` | Ajuster la temperature |
| `/root <chemin>` | Modifier le tool_root pour les envois suivants |
| `/lang ja\|en\|fr` | Changer la langue de l'UI et des prompts |
| `/find <requete>` | Filtrer les messages dans le panneau courant |
| `/help` | Afficher toutes les commandes |

---

## Securite

`127.0.0.1` uniquement par defaut. L'execution shell est reelle — gardez les approbations activees.

Les chemins des outils fichiers sont valides par rapport a `tool_root` a chaque appel : les chemins absolus hors `tool_root` et tout composant `..` sont rejetes en erreur (jamais silencieusement).

Si vous l'exposez sur un reseau, ajoutez une authentification et durcissez l'execution d'outils.

---

## Depannage

**"Failed to connect to github.com via 127.0.0.1"** — proxy mort dans les variables d'environnement :
```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

**Push sans invite interactive** (WDAC / sh.exe Win32 error 5) :
```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

**Push via SSH sur le port 443** (reseau d'entreprise) :
```powershell
.\scripts\push_ssh.ps1
```

**"access denied" sur obstral.exe** — binaire encore en cours d'execution :
```powershell
.\scripts\kill-obstral.ps1
```

---

## Licence

MIT
