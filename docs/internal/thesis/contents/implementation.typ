#import "../lib.typ": *
#import "/packages.typ": *
#import packages.codly: *

= Implémentation

== Orchestration et réconciliation
=== Communication inter-processus
- les différents composants communique entre eux via gRPC
- gRPC est uniquement utilisé comme protocle de transport et client/server car
    il permet aisément de connecter deux processus et de définir des procédures
- il n'est pas utilisé pour définir la structure des messages
- gRPC marque tout les champs comme optionel, cela ce traduit en une structure
    de données dont chaque champ est une `Option<T>` en Rust
- Il serait donc nécessaire de retranscrir ces DTO vers des structure Rust avec
    aucun champ optionel pour les besoins du système.
- Vu que les processus des deux cotés sont en Rust, il est plus simple de
    directement serializer les données dans un format binaire et les
    déserializer dans la structure finale; tous les messages n'ont donc qu'un
    seul champ "raw" qui sont les données serialisées.
- Chaque controleur implémente le service `Reconciler` qui contient deux
    procédure: `Validate` et `Reconcile`
- Le state manager implémente deux services: `StateServer` qui permet a un
    contrôleur de notifier un événement externe nécesitant a une ressoruce
    d'être réconciliée (via `ReconcileNow`)
- il implémente aussi le `ApiService` qui correspond à toutes les procédures que
    le client d'administration peut effectuer (`PushConfig`, `ListResources`,
    ...)

=== Validation d'une ressource
La validation d'une ressource s'effectue à travers la procédure `validate()`. La
requête contient d'abord la nouvelle spécification de la ressource, et si cette
ressource existe déjà, la spécification courrante de celle-ci ainsi que son
état. La ressoure actuelle est transmise car certaines ressources, tel que la
ressource permettant d'installer le système sur le disque, sont toute ou en
partie immutable, et leur modification devrait passer par la recréation d'une
nouvelle ressource. Lors de la validation, le contrôleur peut fournir un
"spécification dérivé" ou précalculé, se basant uniquement sur la spécification
de la ressource. La spécification dérivé n'est mise a jour uniquement lorsque la
spécification change, il n'est pas possible de mettre à jour la spécification
dérivée lors d'une réconciliation normale.

La validation d'une ressource s'effectue a chaque fois qu'une nouvelle
spécification est crée ou qu'une est mise a jour. La validation s'effectue aussi
de manière parallèle.

=== Réconciliation
La réconciliation est une tâche de fond qui va, en boucle, récupérer tous les
identifiants arrivés à échéance puis faire une requête au controlêur responsable
afin de réconcilier la ressource. La réponse inclue le nouvel état, status, tous
les enfants, et toutes les dépendances. En ce qui concerne les dépendences,
uniqueement leur identifiant est transmis. Pour les enfants, leur identifiant et
leur spécification sont transmis.

=== Fonctionnement général de la file d'attente

Comme indiqué dans le #full-ref(<ch:system-design:scheduling>), la boucle de
réconciliation dépend d'une file d'attente, représentée ici par la structure
`Queue<K>`. Cette file est implémentée de manière générique: elle permet de
planifier n'importe quel élément de type `K` à un moment précis. Dans le cadre
du présent système, `K` représente l'identifiant unique d'une ressource. Son
état interne est principalement inclu dans la structure `QueueInner<K>`:

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
    caption: [
        Encapsulation de la file d'attente pour les opérations d'écriture
    ],
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
=== Configuration du noyeau
- le noyeau utilise le defconfig pour l'architecture cible
- par dessus cette config par défaut divers options située dans
    config/common.conf sont ajoutée
- Il est possible de passer plusieurs fragments de configuration. Une fonction
    Nix aété crée et permet de fusionner tous les fragments ensemble avec la
    configuration par défaut. Cela permet de stoquer uniquemetn les changements
    dans le repo
- Pour faciliter la modification de la configuration, un applet Nix permet de
    merge tous les fragmetns, se mettre dans le menuconfig du noyeau, puis
    lorsqu'on en sort, automatiquemetn faire la diff et retourner non seulement
    les options ajoutée/supprimée par rapport a la defconfig (`config.merged`),
    mais aussi par rapport a la config custom (`config.diff`) et aussi la config
    complete (`config.full`)

=== Génération de l'image du système
- L'image final du système (que ce soit l'ISO ou l'image disque brut) est
    assemblée entièremetn dans Nix
- Chaque `crate` en Rust correspond à une `output` dans Nix, ces crates sont
    mises ensemble dans une output nommé `rootfsEnv` qui contient aussi les
    binaires additionelles tel que Podman.
- La particularié de cela est que cela génère un dossier avec tout dans
    `/nix/store` // TODO: Plus de détails sur pourquoi?
- Une output `rootfs` prend cette environement et ajoute les liens symboliques
    de sorte a ce que `/bin` contienne divers liens symboliques vers
    `/nix/store`. Cela sort au final une archive squashfs
- En outre deux autre output sont utile
    - `kernel` qui permet de construitre la bzIamge du noyeau avec les divers
        options de configuration
    - `initrd` qui fournit le système de fichier initiale
- A partir de ces 3 output (`rootfs`, `initrd`, `kernel`), l'output `iso`
    assemble le tout dans une image ISO qui peut être lancée via QEMU ou sur un
    système physique
- L'assemblage pour d'autre architecture est similaire.
- Il est aussi souhaitable d'avoir une image disque brute (par exemple pour les
    systèmes ne pouvant pas démmarer sur une image ISO et nécessitant d'e^tre
    flashé, tel que les RPi); dans ce cas une output spécifique a chaque système
    est crée.
- Dans le cas des RPi, c'est `rpi-sd-image` qui contient les divers éléments
    spécifique pour le Raspberry Pi, et sort une image qui peut directement être
    flashé sur la carte SD

=== Démarrage du système
Lors du démarrage, le bootloader va charger le noyeau ainsi que un système de
fichier initial en mémoire. Une fois que le noyeau a terminé de démarré, ce
dernier va lancer le processus d'initialisation (`/init`) se trouve sur le
système de fichier initial. Ce processus d'initialisation et tout ce qui en
découle consiste le périmètre de l'OS. Le système de ficheir initial étant
chargé en RAM, il est nécessaire que celui-ci soit léger; de fait il ne contient
que le processus d'initialisation. Le but de celui-ci est de trouver le système
de fichier racine complet et de le mettre en place. Ce système de fichier prend
la forme d'une archive SquashFS qui le rend immuable. Dans le cadre d'une image
ISO, cette archive se trouve à côté de l'`initrd` et dans le cas d'une
installation complète, cette archive se trouvera sur la partition de boot. Cette
archive contient l'ensemble du fichier racine avec les différents programes et
dossier. Étant donné que ce système de fichier est immuable, un système de
fichier temporaire est assmeblé par dessus via OverlayFS afin de permettre
l'écriture de fichiers. Les fichiers ainsi écrits seront toutefois supprimés
lors d'un redémarrage, c'est pourquoi ceux-ci sont géré de manière déclarative.

Ce nouveau système de ficheir racine contient lui aussi un processus
d'initialisation, nommé "superviseur". Ce processus est chargé de trouver et
monter le système de fichier surlequel se trouve la configuration du système. Il
va ensuite lancer les divers contrôleurs (les processus permettant d'effectuer
la réconciliatoin), ainsi que l'orhcestrateur de réconciliation.

Lors du démarrage de l'orchestrateur de réconciliation, celui-ci va tenter de
charger la configuration. Si celle-ci n'est pas disponible, il charge une
configuration initiale. Dans les deux cas, l'ensemble des éléments de
configuration sont ajouté à la file d'attente de réconciliation.

- Parler du démarrage, comment les choses sont chargée, et commetn la config est
    chargée

=== Installation
- Parler de comment l'installation système se déroule

=== Sécurité
- Capabilities
- Namespacing
- Authentication (y.c. API)

== Environement de développement

=== Choix du langage
- Rust a été choisi comme language principal pour l'implémentation de ce projet
- Il fallait un language dans lequel il est possible d'aisément faire des appels
    systèmes. Parmis les languages connus il y avait C, C++, Go, et Rust
-

=== Composants et organisation
- Organisation des crates (assez rapide)
- Choix sur les paramètres de compilation en Rust + compiler nightly
- Dépendacnes (surtout le fait qu'on les minimise)

=== Système de build
- L'Environement de build repose sur Nix
- Il est important de distinguer:
    - Nix le système de build
    - Nix le gestionnaire de paquets
    - Nix l'OS
- Il s'agit ici bien de Nix en tant que système de build, et en partie comme
    gestionnaire de paquet.
- L'intérêt ici est de fournir un environement stable entre la machine locale et
    l'environement CI/CD.
- En outre nix permet non seulement de build, mais aussi de rentrer facilemetn
    dans un environement de build ou d'exécuter certains commandes dans un
    environemetn spécial.
- Enfin Nix permet d'avoir des build reproducible bit pour bit, ce qui était une
    propriété désirée pour le projet, même si pas indispensable
- En outre Nix permet de plus facilement faire de la cross-compilation

=== Environement de test
- 3 types de test: unitaire, intégration, et end-to-end
- les tests unitaire sont trivial; il n'intéragissent pas avec le monde
    exterieur ou inétragissent de manière limitée. On utilise le système de test
    classique de Rust (\#[test])
- les tests d'intégration sont plus compliqués: il s'agit par exemple des tests
    du contrôleur réseau. Dans ce cas il n'est pas possible de "juste" les
    exécuter sur l'hôte juste comme ça vu que sa tocuhe a la config réseau. D'un
    autre côté, devoir exécuter l'OS entier pour y tester est compliqué et il
    peut y avoir plein de failure point (en plus que certains truc internes ne
    sont pas exposé à l'OS donc compliqué de tester). Pour ce faire une crate
    `isolate` a été crée qui permet de lancer un test dans un processus séparé
    (via fork) et isolé via des namespaces. Cela permet de fournir un
    environemetn complètement vièrge pour tester avec 0 interfaces réseau, et un
    système de ficheir racine. L'avantage c'est que quand el test est fini, tout
    est détruit. Le problème c'est que vu que c'est un système de ficheir
    complètement vièrge, et bien on peut difficilemnt tester les trucs qui ont
    besoins de binaires externes. Les tests se font toujours via la macro
    \#[test] de Rust et son localisé avec le code testé
- les deux premier tests peuvent se faire sans avoir a (re)build tout le système
    vu qu'il ne repose que sur Rust
- enfin les tests e2e execute l'OS entier et une suite d'actions/update de
    config/etc. Ces tests sont placés dans une crate spéciale "e2e-tests". Étant
    donné qu'il nécessite l'OS entier (et donc l'image de l'OS) pour tourner,
    ils ne peuvent pas directement être exécuté via la commande de test de Rust,
    mais doivent passer via le système de build complet.
- Tous les test Rust sont exécuté via cargo nextest; pour les tests unitaire ce
    n'est pas nécessaire mais pour les tests d'intégration, oui. En raison de la
    nature de l'isolation des tests, lorsque l'un de ceux-ci échoue, cela génère
    un EXIT et le système de test standard s'arrête net, alors que nextest
    fonctionne toujours vu qu'il exéecute chaque test dans un thread et gère les
    exit des threads.

=== Pipeline CI/CD
- Linting (clippy, fmt, etc)
- Parler de comment la doc est build (vraimetn en mention annexe dans la CI je
    pense)
- Parler de la CI et du fait qu'elle exécute tout bien correctement
- Parler des Justfile (alternative au makefile):
    - en gros c'est un peu chiant de devoir build nix a chaque foit qu'on veut
        lancer une commande, donc une fois qui'on a fait nix develop, on peut
        juste faire "just check" pour lancer clippy
    - et côté CI, et bien il sufffit de faire "nix develop -c 'just check'"
