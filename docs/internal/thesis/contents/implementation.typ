#import "../lib.typ": *

= Implémentation

// TODO: Amorcer le chapitre

// TODO: Parler de l'implémentation spécifique de certains contrôleurs?
// Par exemple le contrôleur system/static-file ou network/route ou network/dhcp

== Environement de développement
=== Choix du langage
Rust est choisi comme langage principal pour l'implémentation du projet. Ce
choix repose sur la nécessité de disposer d'un langage permettant d'effectuer
aisément des appels système, contrainte partagée par le C, le C++ et le Go,
langages également considérés. Le choix final se porte sur Rust en raison de la
maîtrise préalable de ce langage, acquise avant le début du projet. Le langage
Go est par ailleurs utilisé pour la partie du projet reposant sur Terraform, cet
usage étant imposé par une contrainte propre à cet outil plutôt que par un choix
indépendant.

=== Composants et organisation
Le code du projet est réparti entre deux répertoires principaux selon le langage
utilisé: `rust/`, regroupant l'ensemble du code Rust, et `go/`, regroupant le
code Go utilisé pour la partie du projet reposant sur Terraform. Ces deux
répertoires constituent respectivement la racine du workspace Rust et celle du
workspace Go, chacun regroupant l'ensemble des modules propres à son langage
sous une configuration de compilation commune. Le répertoire `rust/` contient
lui-même deux sous-répertoires: `cmd/`, où sont situés les binaires exécutables,
et `crates/`, où sont situées les bibliothèques internes partagées entre
plusieurs binaires. Le reste de l'arborescence répartit les éléments non liés au
code applicatif: la configuration du noyau et les éléments annexes sont situés
dans `linux/`, les définitions de services et de messages gRPC dans `proto/`, et
les documents propres au travail de diplôme dans `docs/internal`.

La compilation s'effectue via le compilateur nightly de Rust. Ce choix est
motivé par le recours à la fonctionnalité `build-std`, permettant la
reconstruction de la bibliothèque standard avec les mêmes options d'optimisation
que le reste du projet, ce qui contribue à la minimisation de la taille des
artefacts finaux, ainsi que par l'utilisation de règles de linting propres au
canal nightly. Le qualificatif nightly, ou instable, désigne l'absence de
garantie de pérennité des fonctions, options et autres éléments exposés par ce
canal, ces derniers pouvant être modifiés ou retirés sans préavis. Cette
instabilité porte uniquement sur la stabilité de l'interface exposée dans le
temps, et non sur la fiabilité d'exécution de la fonctionnalité elle-même.

Le nombre de dépendances externes est volontairement minimisé, cette contrainte
s'appliquant également aux dépendances retenues, elles-mêmes choisies pour leur
simplicité. Cette minimisation répond à une volonté de comprendre en profondeur
le fonctionnement du système, plutôt que de déléguer une part importante de son
comportement à des bibliothèques tierces.

=== Système de build
L'environnement de build repose sur l'outil Nix. Une distinction est requise
entre trois usages du terme "Nix": Nix en tant que système de build, Nix en tant
que gestionnaire de paquets, et Nix en tant que distribution Linux (NixOS). Seul
le premier usage, complété partiellement par le second, est utilisé dans le
cadre de ce projet.

Le recours à Nix vise à fournir un environnement stable entre la machine de
développement locale et l'environnement d'intégration continue. Nix permet non
seulement la construction des artefacts, mais aussi l'entrée dans un
environnement de build ou l'exécution de commandes au sein d'un environnement
isolé. La reproductibilité bit à bit des builds, permise par Nix, constitue une
propriété désirée pour le projet, bien que non indispensable. Nix facilite en
outre la mise en œuvre de la cross-compilation, soit la compilation depuis une
architecture CPU particulière vers une architecture différente, par exemple de
x86 vers ARM64.

L'ensemble de la chaîne de build du système est pris en charge par Nix: le
noyau, les différentes crates composant le projet, ainsi que les artefacts
finaux assemblés à partir de ces éléments. Le détail de cet assemblage est
développé ultérieurement dans ce chapitre.

=== Environement de test
Trois catégories de tests sont mises en œuvre: les tests unitaires, les tests
d'intégration, et les tests de bout en bout (end-to-end). Les tests unitaires
sont dépourvus de complexité particulière, n'interagissant pas ou peu avec
l'environnement extérieur. Ces tests sont réalisés à l'aide du système de test
standard de Rust, via la macro `#[test]`.

Les tests d'intégration présentent une complexité supérieure. Le test du
contrôleur réseau en constitue un exemple représentatif: une exécution directe
sur l'hôte est exclue, la configuration réseau de ce dernier étant affectée,
tandis qu'une exécution au sein du système complet introduirait de nombreux
points de défaillance supplémentaires, certains éléments internes n'étant par
ailleurs pas exposés à ce niveau. Une crate nommée `isolate` est créée à cet
effet, permettant l'exécution d'un test dans un processus séparé, via un appel
`fork`, isolé par des namespaces Linux. Un environnement entièrement vierge est
ainsi fourni pour chaque test, dépourvu d'interface réseau et disposant d'un
système de fichiers racine propre; l'ensemble de cet environnement est détruit à
l'issue du test. Cette isolation complète empêche cependant le test de binaires
externes non présents dans l'environnement vierge ainsi constitué. Ces tests
sont, comme les tests unitaires, réalisés via la macro `#[test]`, et localisés à
proximité du code testé.

Les tests unitaires et les tests d'intégration partagent la propriété de ne pas
nécessiter la reconstruction du système complet, ne reposant que sur
l'écosystème Rust. Ces deux catégories de tests peuvent ainsi être exécutées
directement au sein de l'environnement de développement Nix, la commande de test
standard mobilisant la compilation incrémentale de Rust plutôt qu'une
reconstruction complète.

Les tests de bout en bout exécutent le système complet ainsi qu'une suite
d'actions et de mises à jour de configuration. Ces tests sont regroupés dans une
crate dédiée, nommée `e2e-tests`. Cette catégorie de test vise à reproduire les
conditions d'utilisation finale du système, ce qui nécessite l'exécution de
l'image ISO complète plutôt que du seul code applicatif. L'exécution de ces
tests repose sur le lancement d'une machine virtuelle via QEMU, à partir de
cette image ISO. Cette dépendance à l'image complète empêche l'exécution directe
de ces tests via la commande de test standard de Rust; leur exécution passe par
le système de build dans son intégralité.

L'ensemble des tests Rust est exécuté via l'outil `cargo nextest`. Ce recours
n'est pas strictement nécessaire pour les tests unitaires, mais s'avère requis
pour les tests d'intégration. L'isolation propre à ces derniers provoque, en cas
d'échec, un appel système `exit` qui interrompt immédiatement le système de test
standard de Rust, alors que `cargo nextest` poursuit son exécution, chaque test
étant exécuté dans un thread distinct dont la terminaison est gérée
indépendamment.

=== Pipeline CI/CD
L'ensemble du projet reposant sur Nix, la pipeline CI/CD (#repo(
    ".gitlab-ci.yml",
)) en hérite également, ce qui simplifie sa mise en œuvre: les étapes exécutées
par la pipeline se limitent, dans une large mesure, à l'invocation de commandes
Nix génériques, indépendantes de l'environnement d'exécution. L'ordonnancement
des tâches repose sur GitLab CI). Le recours à un runner personnalisé est
requis, les runners institutionnels ne permettant ni l'utilisation de Nix ni la
virtualisation imbriquée, requise pour l'exécution des tests de bout en bout, et
disposant par ailleurs de ressources de calcul limitées.

La pipeline est déclenchée à chaque push sur le dépôt, entraînant l'exécution de
l'ensemble des étapes suivantes, dans l'ordre. Une phase de linting est exécutée
en premier lieu, portant à la fois sur la documentation et sur le code. Une
phase de test lui succède, structurée séquentiellement: les tests unitaires et
les doctests de Rust#footnote[
    Un doctest est un exemple de code intégré à la documentation d'une fonction
    ou d'un module, compilé et exécuté automatiquement lors de l'exécution des
    tests afin de garantir la validité des exemples fournis @bib-rust-doctest.
] sont exécutés en parallèle, suivis des tests d'intégration. La phase de build
est ensuite exécutée, suivie enfin des tests de bout en bout. L'ensemble du
processus est illustré dans la~#figure-num-ref(<cicd>):

#include "../diagrams/cicd.typ"

Le linting couvre deux aspects distincts: le formatage et l'analyse statique. Le
formatage est vérifié via `typstfmt` pour la documentation, `buf` pour les
définitions Protocol Buffers, et `fmt` pour le code Rust. L'analyse statique
repose sur `buf` pour les définitions Protocol Buffers et sur `clippy` pour le
code Rust. Les options les plus strictes de `clippy` sont activées, interdisant
notamment le recours à `unwrap` ou à des constructions équivalentes, ainsi que
l'utilisation de `println`, afin de garantir que toute sortie transite par le
système de journalisation. Diverses règles additionnelles sont par ailleurs
activées afin d'assurer l'homogénéité du code. Toute désactivation ponctuelle
d'une règle doit être accompagnée d'une justification explicite, mécanisme
nativement supporté par `clippy`; une exception dépourvue de justification est
rejetée par la pipeline. Une exception générale est toutefois appliquée au code
de test, pour lequel le recours à `unwrap` est autorisé, cette construction
constituant la méthode recommandée pour exprimer une assertion dans ce contexte.

L'exécution répétée de commandes au sein de l'environnement Nix s'avérant peu
pratique lors du développement courant, un outil `Justfile`, alternative au
`Makefile`, est mis en place. La distinction fondamentale entre développement et
intégration continue réside dans le mode d'invocation de Nix: le développement
s'effectue à l'intérieur d'un environnement interactif ouvert via `nix develop`,
dans lequel une commande telle que `just check` peut être directement invoquée
pour exécuter le linting. La pipeline d'intégration continue, à l'inverse,
n'ouvre pas un tel environnement interactif; chaque commande y est invoquée
depuis l'extérieur, via `nix develop -c "just
check"`, ce qui évite la reconstruction de l'environnement à chaque étape tout
en conservant sa reproductibilité.

== Orchestration et réconciliation
=== Communication inter-processus
Les composants communiquent entre eux via le protocole gRPC. Ce dernier est
utilisé uniquement comme mécanisme de transport et de connexion client-serveur,
permettant la connexion de deux processus et la définition de procédures. La
structure des messages n'est en revanche pas définie par ce protocole.

Le protocole gRPC marque chaque champ comme optionnel, ce qui se traduit par une
structure de données dont chaque champ correspond à un `Option<T>` en Rust. Une
retranscription de ces DTO vers des structures Rust sans champ optionnel est
donc requise. Les processus des deux côtés étant implémentés en Rust, les
données sont directement sérialisées dans un format auto-descriptif, JSON en
l'occurrence, puis désérialisées dans la structure finale; chaque message ne
contient ainsi qu'un unique champ nommé "raw", correspondant aux données
sérialisées. Ce choix est motivé par le caractère partiellement générique de la
structure transmise, qui impose la présence d'un schéma auto-descriptif. Les
messages étant échangés entre processus locaux et de taille réduite, le gain de
performance apporté par un format binaire est jugé marginal.

Chaque contrôleur implémente le service `Reconciler`, comportant deux
procédures: `Validate` et `Reconcile`. Le state manager implémente deux
services. Le service `StateServer` permet à un contrôleur de notifier un
événement externe nécessitant la réconciliation d'une ressource, via la
procédure `ReconcileNow`. Le service `ApiService` regroupe l'ensemble des
procédures accessibles au client d'administration, telles que `PushConfig` et
`ListResources`.

=== Validation d'une ressource
La validation d'une ressource s'effectue à travers la procédure `validate()`. La
requête contient d'abord la nouvelle spécification de la ressource, et si cette
ressource existe déjà, la spécification courante de celle-ci ainsi que son état.
La ressource actuelle est transmise car certaines ressources, telle que la
ressource permettant d'installer le système sur le disque, sont totalement ou
partiellement immuables; leur modification doit alors passer par la recréation
d'une nouvelle ressource.

Le champ d'état est optionnel dans la requête de validation, son absence
correspondant au cas d'une première réconciliation, pour laquelle aucun état
antérieur n'existe encore. À l'inverse, certains champs de la spécification
peuvent être dérivés intégralement à partir de la spécification elle-même, sans
intervention de l'utilisateur. Le contrôleur fournit alors, en réponse à la
validation, une spécification dite dérivée, calculée uniquement à partir de la
spécification soumise. Cette spécification dérivée n'est mise à jour que lorsque
la spécification change; il n'est pas possible de la modifier lors d'une
réconciliation normale. Ce mécanisme dispense l'utilisateur de renseigner
manuellement des champs entièrement déductibles, tout en évitant un recalcule
systématique de ces champs à chaque réconciliation. La garantie d'existence de
la spécification dérivée à chaque réconciliation permet en outre au contrôleur
de traiter ce champ comme systématiquement disponible, ce qui allège la logique
de réconciliation en dispensant celle-ci de toute vérification de présence.

La validation d'une ressource s'effectue à chaque création ou mise à jour d'une
spécification. Lorsque plusieurs ressources sont ajoutées ou modifiées
simultanément, l'ensemble de ces ressources est validé en parallèle. Une erreur
affectant un seul élément de cet ensemble entraîne l'échec de la validation pour
la totalité des ressources concernées.

=== Réconciliation
La réconciliation est une tâche de fond qui récupère, en boucle, l'ensemble des
identifiants arrivés à échéance, puis transmet une requête au contrôleur
responsable afin de réconcilier chaque ressource. Cette requête reprend la
structure de ressource complète, décrite dans le @code-resource-def:

#figure(
    label: <code-resource-def>,
    caption: [Structure d'une ressource transmise lors de la réconciliation],
    source: link("cmd/state-manager/src/..."), // TODO: Verify link
    ```rust
    pub struct Resource<Spec, DerivedSpec, State> {
        pub id: Identity,
        pub phase: Phase,
        pub status: Status,
        pub spec: Spec,
        pub derived_spec: DerivedSpec,
        pub state: Option<State>,
        pub children: Vec<TerminalResource<Value, Value, Value>>,
        pub dependencies: Vec<TerminalResource<Value, Value, Value>>,
        pub dependents: Vec<TerminalResource<Value, Value, Value>>,
    }
    ```,
)

Dans le @code-resource-def, les champs `children`, `dependencies` et
`dependents` reposent sur le type `TerminalResource`, une variante de `Resource`
dans laquelle ces mêmes champs sont réduits à de simples identifiants, ce qui
évite la récursion de la structure.

Le message de réconciliation contient une ressource sous forme générique. Le
contrôleur désérialise d'abord cette ressource générique afin d'en identifier le
schéma, puis désérialise les champs génériques dans la structure finale
correspondante. La réconciliation d'une ressource est déclenchée toutes les
trente secondes, ou plus tôt si le contrôleur notifie le `state-manager` d'une
nécessité de réconciliation anticipée.

Les ressources arrivées à échéance sont réconciliées séquentiellement, dans
l'ordre de leur ancienneté, sans regroupement en lots. Deux ressources
consécutives destinées au même contrôleur font ainsi l'objet de deux requêtes
distinctes, la seconde n'étant transmise qu'après réception de la réponse à la
première. Un échec affectant la réconciliation d'une ressource n'empêche pas la
tentative de réconciliation de la ressource suivante, sauf lorsque cette
dernière dépend de la première, auquel cas son échec devient probable sans être
garanti.

La réponse du contrôleur inclut le nouvel état, le status, l'ensemble des
enfants, et l'ensemble des dépendances, dont la structure est décrite dans le
@code-response-def:

#figure(
    label: <code-response-def>,
    caption: [Structure de réponse d'une réconciliation],
    source: link("cmd/state-manager/src/..."), // TODO: Verify link
    ```rust
    pub struct ResourceResponse<State> {
        pub status: Status,
        pub state: Option<State>,
        pub children: Vec<SubResource<GenericSpecification>>,
        pub dependencies: HashSet<Identity>,
    }
    ```,
)

// TODO: Commenter un peu plus le code?

Seul l'identifiant est transmis pour les dépendances, tandis que l'identifiant
et la spécification sont transmis pour les enfants.

Deux catégories d'erreur sont distinguées lors de la réconciliation: l'erreur de
protocole et l'erreur de logique. Une erreur de protocole survient lorsque les
données échangées via gRPC sont invalides ou non interprétables, ou lorsqu'une
erreur imprévue se produit au niveau du transport. Une erreur de logique, en
revanche, correspond à un échec géré par le contrôleur dans le cadre normal de
la réconciliation, et se traduit par un status d'erreur porté dans le champ
`status` de la réponse. Dans ce dernier cas, la réconciliation est considérée
comme réussie du point de vue du protocole, la présence d'une réponse
correctement formée suffisant à cette qualification, indépendamment du contenu
du status. En cas d'erreur de protocole, le `state-manager` attribue lui-même à
la ressource un status d'erreur qualifié de "transport"; dans tous les autres
cas, le status attribué à la ressource correspond à celui fourni par le
contrôleur dans sa réponse.

=== Fonctionnement général de la file d'attente
Comme indiqué dans le #full-ref(<ch:system-design:scheduling>), la boucle de
réconciliation dépend d'une file d'attente, représentée ici par la structure
`Queue<K>`. Cette file est implémentée de manière générique: elle permet de
planifier n'importe quel élément de type `K` à un moment précis. Dans le cadre
du présent système, `K` représente l'identifiant unique d'une ressource. Son
état interne est principalement inclut dans la structure `QueueInner<K>`:

#figure(
    caption: [Structure interne de la file d'attente],
    source: link("cmd/state-manager/src/queue.rs"), // TODO: Verify link
    ```rust
    struct QueueInner<K> {
        scheduled: HashMap<K, Instant>,
        queue: BTreeMap<Instant, HashSet<K>>,
    }
    ```,
)

Les ressources ainsi planifiées sont stockées dans un dictionnaire basé sur un
B#{ sym.hyph.nobreak }arbre~(`BTreeMap`)~@bib-rust-std-btreemap, indexé par
l'instant auquel leur réconciliation est prévue. Le recours à un B-arbre
facilite la récupération des ressources dont la planification est arrivée à
échéance: la date de réconciliation étant une valeur numérique ordonnée, toutes
les valeurs inférieures à l'instant présent correspondent à des échéances
passées et peuvent être traitées. L'implémentation Rust des B-arbres permet en
outre de scinder l'arbre en deux à partir d'une clé donnée, ce qui correspond
précisément au cas d'utilisation recherché. En outre, l'arbre étant ordonné,
cela permet de parcouris les éléments en commencant par le plus ancien, sans
recourir à une étape de tri additionnelle.

La valeur associée à une clé n'est pas un identifiant unique, mais un ensemble
d'identifiants. En effet, lorsqu'une nouvelle configuration est soumise,
l'ensemble des ressources nouvellement définies doit être planifié simultanément
pour réconciliation; c'est pourquoi chaque clé temporelle est associée à un
ensemble de valeurs uniques~(`HashSet`)~@bib-rust-std-hashset plutôt qu'à un
identifiant unique. Un champ supplémentaire, `scheduled`, complète cette
structure en fournissant un index inversé permettant de déterminer rapidement
si, et à quel moment, une ressource donnée est planifiée.

Cette file expose la méthode asynchrone `drain_expired()`, qui attend qu'une ou
plusieurs clés arrivent à échéance avant de les retourner. L'attente est
passive: la fonction calcule le délai jusqu'à la prochaine échéance connue, puis
se met en pause jusqu'à ce délai. Pour gérer le cas où un élément serait
planifié pendant cette attente, avec une échéance plus proche que celle déjà
calculée, ou alors qu'aucune échéance n'existait, un canal de notification est
utilisé, basé sur la structure `Notify` de la runtime asynchrone "Tokio".

Étant donné que plusieurs opérations d'écriture (ajout individuel, ajout en
masse, replanification) peuvent modifier l'échéance la plus proche et donc
nécessiter une notification, ces opérations passent par un `QueueInnerGuard`:

#figure(
    caption: [Encapsulation de la file d'attente pour les opérations
        d'écriture],
    source: link("cmd/state-manager/src/queue.rs"), // TODO: Verify link
    ```rust
    struct QueueInnerGuard<'a, K> {
        guard: RwLockWriteGuard<'a, QueueInner<K>>,
        notify: Arc<Notify>,
        earliest: Option<Instant>,
    }
    ```,
)

Au moment de sa construction, ce guard capture l'échéance la plus proche connue
avant modification. Lorsqu'il est détruit (`Drop`~@bib-rust-std-drop,
l'équivalent Rust d'un `defer` en Go ou d'un destructeur en programmation
orientée objet), il compare cette échéance initiale à l'échéance courante et
déclenche une notification si celle-ci a changé, que ce soit parce qu'une
échéance plus proche a été introduite, ou parce que la file est passée d'un état
non vide à vide ou inversement. Les opération de suppression ne passent en
revanche pas par ce système car #todo-inline[à justifier].

Enfin, la file est encapsulée dans une structure `Queue`, qui fournit un accès
concurrent aux différentes méthodes:

#figure(
    caption: [Encapsulation de la file d'attente pour un accès concurrent],
    source: link("cmd/state-manager/src/queue.rs"), // TODO: Verify link
    ```rust
    struct Queue<K> {
        queue: Mutex<QueueInner<K>>,
        notify: Arc<Notify>,
    }
    ```,
)

L'accès à `QueueInner` est protégé par un~`Mutex`~@bib-rust-std-mutex, chaque
opération exposée par `Queue` nécessitant de toute façon un accès exclusif à la
structure sous-jacente.

// TODO: Plus de détails?

== Environement d'exécution du système
=== Configuration du noyau
La configuration du noyau repose sur la configuration par défaut propre à
l'architecture cible, complétée par un ensemble d'options supplémentaires
situées dans le fichier `config/common.conf`. Plusieurs fragments de
configuration peuvent être fournis simultanément; une fonction Nix est créée à
cet effet, permettant de fusionner l'ensemble de ces fragments avec la
configuration par défaut. Cette approche permet de ne stocker, dans le dépôt,
que les changements apportés à la configuration par défaut, plutôt que la
configuration complète.

// TODO: préciser les options supplémentaires notables ajoutées dans config/common.conf, et la raison de leur activation.

Une commande est mise à disposition afin de faciliter la modification de cette
configuration. Cette commande fusionne l'ensemble des fragments, ouvre
l'interface `menuconfig` du noyau, puis calcule automatiquement, à la sortie de
cette interface, la différence entre la configuration modifiée et trois
références distinctes. La différence par rapport au `defconfig` est exposée sous
`config.merged`, la différence par rapport à la configuration personnalisée sous
`config.diff`, et la configuration complète sous `config.full`.

=== Génération de l'image du système
L'image finale du système, qu'il s'agisse d'une image ISO ou d'une image disque
brute, est assemblée entièrement au moyen de Nix. Chaque crate Rust du projet
correspond à un output Nix; l'ensemble de ces outputs est regroupé dans un
output nommé `rootfsEnv`, lequel intègre également des binaires additionnels,
tel que Podman. Cet output génère un dossier regroupant l'ensemble de ces
éléments au sein du `/nix/store`, ce dernier constituant le mécanisme par lequel
Nix stocke tout résultat de construction, indépendamment de sa nature, sous un
chemin adressé par le contenu de ce résultat.

Un output `rootfs` reprend cet environnement et y ajoute les liens symboliques
nécessaires, de sorte que les répertoires `/bin`, `/etc`, etc. contiennent des
liens symboliques pointant vers le `/nix/store`. Cet output produit, en sortie,
une archive au format SquashFS. Deux outputs additionnels complètent cet
assemblage: `kernel`, qui construit la `bzImage` du noyau selon les options de
configuration retenues, et `initrd`, qui fournit le système de fichiers initial.

À partir de ces trois outputs, `rootfs`, `initrd` et `kernel`, un output `iso`
assemble l'ensemble en une image ISO, exécutable via QEMU ou sur un système
physique. L'assemblage destiné à d'autres architectures suit une démarche
similaire.

Une image disque brute est par ailleurs requise pour les systèmes ne pouvant
démarrer depuis une image ISO et nécessitant un flashage direct, tel que le
Raspberry Pi. Un output spécifique est créé pour chaque système visé par ce mode
de déploiement. Dans le cas du Raspberry Pi, l'output `rpi-sd-image` regroupe
les éléments propres à cette plateforme et produit une image directement
destinée au flashage sur une carte SD.


=== Démarrage du système
Lors du démarrage, le bootloader charge le noyau ainsi qu'un système de fichiers
initial en mémoire. Une fois le noyau démarré, ce dernier lance le processus
d'initialisation (`/init`), situé sur ce système de fichiers initial. Ce
processus d'initialisation, ainsi que l'ensemble des opérations qui en
découlent, constitue le périmètre de l'OS. Le système de fichiers initial étant
chargé en RAM, sa légèreté est requise; il ne contient, à ce titre, que ce seul
processus d'initialisation. Ce dernier a pour rôle de localiser le système de
fichiers racine complet et de le mettre en place. Ce système de fichiers prend
la forme d'une archive SquashFS, ce qui le rend immuable. Dans le cadre d'une
image ISO, cette archive se trouve à proximité de l'`initrd`; dans le cas d'une
installation complète, elle se trouve sur la partition de démarrage. Cette
archive contient l'ensemble du système de fichiers racine, avec les différents
programmes et répertoires. Ce système de fichiers étant immuable, un système de
fichiers temporaire est superposé au moyen d'OverlayFS, afin de permettre
l'écriture de fichiers. Les fichiers ainsi écrits sont supprimés lors d'un
redémarrage, ce qui garantit un système identique à son état d'origine à chaque
redémarrage.

#include "../diagrams/sysinit.typ"

La #figure-num-ref(<sysinit>) illustre les étapes principales de ce démarrage.
L'étape initiale, prise en charge par le superviseur, consiste à localiser et
monter la partition racine, puis à constituer l'environnement de base du
système, incluant les points de montage `/dev` et `/proc` ainsi que l'interface
réseau locale. Le superviseur localise ensuite et monte une configuration
minimale, dite configuration précoce. Le contrôle est alors transféré au
processus core-controller, qui localise, déchiffre et monte la configuration
complète ainsi que l'état persistant du système, avant de démarrer
successivement le contrôleur réseau, l'API, puis les autres contrôleurs du
système. La réconciliation ne débute qu'une fois l'ensemble de ces contrôleurs
prêts.

#include "../diagrams/procstart.typ"

La #figure-num-ref(<procstart>) détaille l'arbre d'exécution des processus ainsi
démarrés, en distinguant les processus internes au projet des processus tiers.
Le processus `init` démarre le superviseur, lequel démarre à son tour le
core-controller. Ce dernier démarre l'API ainsi que l'ensemble des contrôleurs,
dont le contrôleur de conteneurs et le contrôleur réseau. Le contrôleur de
conteneurs démarre à son tour l'environnement d'exécution de conteneurs tiers
retenu, tel que Podman, tandis que le contrôleur réseau démarre les clients DHCP
et NTP.

Ce nouveau système de fichiers racine contient, à son tour, un processus
d'initialisation, nommé superviseur. Ce dernier est chargé de localiser et de
monter le système de fichiers sur lequel se trouve la configuration du système,
puis de lancer les différents contrôleurs, soit les processus chargés
d'effectuer la réconciliation, ainsi que l'orchestrateur de réconciliation.

Lors de son démarrage, l'orchestrateur de réconciliation tente de charger la
configuration existante. En l'absence de configuration disponible, une
configuration initiale est chargée. Dans les deux cas, l'ensemble des éléments
de configuration est ajouté à la file d'attente de réconciliation.


=== Installation
#todo[Installation]
