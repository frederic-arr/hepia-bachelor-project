/*
- [x] Parler de l'IA
- [x] Plus parler du projet de semestre
	- [x] Quel est le but du projet de semesetre
- p11. /var ext4
- p12. légende schémaa

- abus de language NixOS => pas tout recompiler
- parler des challenges
- dire je
*/


#import "/packages.typ": *
#import packages.touying: *
#import themes.metropolis: *

// Filler slides are to be used as the slide before the presentation starts
// or after the presentation ends. The projector should be frozen on those so
// that the we can do our setup in the background.
#let filler-slide(body) = focus-slide[
    #image("/lib/assets/hepia-logo.svg")
    #body
]

#let cntr = counter("touying-slide-counter")

#filler-slide[]

#title-slide()
/*
Bonjour, aujourd'hui j'ai l'immense plaisir de vous présenter mon travail de
bachelor qui s'intitule OS pour le déploiemetn de services conteneurisés.

Toud d'abird, je vais vous présenter la problématique et le contexte de se travail,
ainsi que vous présenter la solution développé avant de passer à la suite.
*/

= Introduction <touying:skip>
== Problématique
#speaker-note[
    - Début: 00:20 #h(5cm) Fin: *01:50*
    #only(<part-a>)[
        - *Ce projet s'intéresse l'administration d'environement modestes*
        - Par modestes, il faut entendre des environnements administrés par une
            seule personne, sans clustering, sans déploiement multi-région, et
            sans redondance particulière.
        - C'est typiquement le genre de déploiement que vous pourriez avoir si
            vous hébergez un site à titre personnel ou un petit projet avec une
            dizaine, une centaine, voire un millier d'utilisateurs tout au plus.
            *>>>*
    ]
    #only(<part-b>)[
        - Modestes, oui, mais aussi modernes: les déploiements visés se basent
            sur la conteneurisation pour gérer l'application.
        - Cette conteneurisation permet de s'affranchir d'une grande partie de
            la complexité dans l'administration de systèmes, à savoir
            l'installation et la gestion des applications elles-mêmes. *>>>*
    ]
    #only(<part-c>)[
        - Toutefois, il reste nécessaire de configurer et de maintenir l'hôte
            sur lequel les conteneurs s'exécutent. *>>>*
        - Cette configuration, dans ce genre de déploiement, suit un ensemble
            assez commun d'étapes, à commencer par l'installation du système
            d'exploitation. *>>>*
        - Il est ensuite nécessaire de le configurer (accès SSH, configuration
            réseau, installation des paquets) *>>>*
        - puis d'y installer l'environnement d'exécution de conteneurs, par
            exemple Docker. *>>>*
        - Une fois tout cela fait, vous pourrez y déployer vos conteneurs. *>>>*
    ]
    #only(<part-d>)[
        - L'administration de l'hôte est simple, presque standardisée, et sa
            finalité est claire. Alors pourquoi aucun système d'exploitation ne
            se limite à faire exactement ça, et rien de plus? *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Déploiements modestes
#waypoint(<part-b>)
- Basé sur les conteneurs
#waypoint(<part-c>)
- Étapes communes
    #pause
    + Installer l'OS
    #pause
    + Configurer le système (SSH, réseau, paquets)
    #pause
    + Installer le runtime de conteneur
    #pause
    + Déployer les conteneurs
#waypoint(<part-d>)
#v(0.5cm)
#box(
    fill: rgb("e2001a").lighten(75%),
    inset: (top: 1cm, rest: 7.5mm),
    radius: 0.5em,
    width: 100%,
)[
    #place(dx: -7.5mm, dy: -1.5cm, box(
        fill: rgb("e2001a"),
        inset: 2.5mm,
        radius: 0.25em,
        strong(text(fill: white)[Constat]),
    ))

    Malgré ces besoins simples, aucun OS ne remplit ce rôle simplement.
]

== Contexte
#speaker-note[
    - Début: 01:50 #h(5cm) Fin: *02:50*
    #only(<part-a>)[
        - Et je me penche sur cette question depuis mi-octobre
        - En effet, avant ce travail, un travail préalable a eu lieu: le projet
            de semestre, entre octobre et mars. *>>>*
        - Ce travail avait pour but premier d'identifier des solutions
            potentielles correspondant aux besoins exprimés; il en ressort
            qu'aucune n'est adéquate. *>>>*
        - Il s'agissait également d'établir les bases de très haut niveau pour
            l'architecture et le fonctionnement *>>>*
        - ainsi que de me renseigner et de choisir les technologies à employer.
            *>>>*
    ]
    #only(<part-b>)[
        - Par ailleurs, tant dans le projet de semestre que dans ce projet, l'IA
            n'a été que très peu utilisée, principalement dans le but de
            déboguer et d'améliorer certains aspects annexes de
            l'implémentation. *>>>*
        - Ces tentatives, cela dit, n'ont pas été couronnées de succès. *>>>*
        - Quoi qu'il en soit, aucun code n'a été produit par l'IA, et aucune
            décision architecturale n'a été prise par elle. *>>>*
    ]
    #only(<part-c>)[
        - Ce choix tient au fait qu'il s'agit avant tout d'un projet personnel
            qui me tient à coeur: je voulais avant tout voir mes compétences à
            l'oeuvre. *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Projet de semestre, octobre #sym.arrow.double mars)
    #pause
    - Identification de solution existante adéquates: *aucune*
    #pause
    - Conceptualisation de très haut niveau
    #pause
    - Recherche des "briques" logicielles
#waypoint(<part-b>)
- IA: tentative pour déboguage + amélioration CI/CD
    #pause
    - Sans succès
    #pause
    - *Pas de code ou de décisions architecturales*
#waypoint(<part-c>)
- Projet personnel


== Solution
#speaker-note[
    - Début: 02:50 #h(5cm) Fin: *04:30*
    #only(<part-a>)[
        - Comme je l'ai dit, aucune solution existante ne convient. Dans ce
            travail, je vais donc vous présenter la solution que j'ai conçue
            pour répondre à ce besoin. *>>>*
        - La solution que je propose est une distribution Linux spécialisée pour
            la conteneurisation. *>>>*
    ]
    #only(<part-b>)[
        - C'est-à-dire une solution aussi minimale que possible, dans le but de
            déployer rapidement et simplement des conteneurs. *>>>*
        - Et par minimale, je vais ici à l'extrême en retirant d'abord toute
            forme d'accès interactif: pas de SSH, pas de shell, et pas de
            commandes. *>>>*
        - Pour remplacer tout cela, le système expose une API. Cette API est
            principalement en lecture seule, à l'exception d'un élément: la
            gestion de la configuration elle-même. *>>>*
    ]
    #only(<part-c>)[
        - Cette configuration passe par un fichier unique: tout y est centralisé
            et défini, que ce soit les conteneurs, le réseau, le stockage, ou
            l'installation. *>>>*
        - Le but est d'avoir un système entièrement déclaratif: vous déclarez
            simplement l'état dans lequel vous voulez que votre système soit, et
            il s'occupe lui-même d'y parvenir, sans que vous ayez à effectuer la
            moindre action. *>>>*
    ]
    #only(<part-d>)[
        - Enfin, un autre point très important est l'homogénéité: l'idée est que
            le même fichier de configuration puisse être utilisé sur plusieurs
            types d'environnements, que ce soit le cloud, le bare-metal, un
            Raspberry Pi, etc. *>>>*
    ]
    #only(<part-e>)[
        - Pour résumer grossièrement le fonctionnement:
            - vous écrivez un fichier de configuration
            - vous le transmettez à l'API
            - puis le système se charge de configurer les conteneurs, le réseau,
                le stockage, etc., selon ce que vous avez exprimé dans votre
                fichier.
        - Pour mieux comprendre comment cela se passe, je vais vous en faire la
            démonstration. *|||*
    ]
]

#grid(
    columns: 2,
    [
        #waypoint(<part-a>, advance: false)
        #pause
        - Distribution Linux spécialisée
        #waypoint(<part-b>)
        - Surface *minimale*: le strict nécessaire pour les conteneurs
            #pause
            - Pas d'SSH, de shell, ou de commandes
            #pause
            - Piloté par API
        #waypoint(<part-c>)
        - Fichier de configuration unique
            #pause
            - Système déclaratif
            #waypoint(<part-d>)
            - Homogène: bare-metal, VPS, embarqué, etc.
    ],
    [
        #waypoint(<part-e>)
        #align(center, image("/assets/image-4.png"))
    ],
)

= Démonstration

= <touying:skip>
== Suite de la présentation
#speaker-note[
    - Début: 04:30 #h(5cm) Fin: *04:50*
    - Dans la suite de cette présentation, je vais d'abord vous présenter les
        concepts fondamentaux de cette solution, ainsi que quelques aspects clés
        de son implémentation.
    - Ensuite, je parlerai de la façon dont la solution a été testée et validée,
        et de la manière dont ses performances ont été mesurées, avant de la
        comparer à deux autres solutions proches des besoins exprimés, à savoir
        Talos Linux et NixOS. *|||*
]

+ *Conception*
+ *Implémentation*
+ *Tests & Validation*
+ *Comparaison avec d'autres solutions*

= Conception
#speaker-note[
    - Début: 04:50 #h(5cm) Fin: *05:00*
    - Et je vais donc maintenant vous présenter les conceptes éssentiels de
        cette solution, à commencer par la réconciliation. *|||*
]


== Réconciliation
#speaker-note[
    - Début: 05:00 #h(5cm) Fin: *06:00*
    #only(<part-a>)[
        - En termes simples, la réconciliation est une boucle qui compare l'état
            actuel avec l'état désiré afin de les faire converger
            *>>>*
        - J'ai schématisé ici le fonctionnement général de cette boucle
        + D'abord, l'état actuel de la ressource à réconcilier est observé et
            capturé
        + Cet état actuel est ensuite comparé avec l'état désiré, c'est-à-dire
            la configuration *>>>*
    ]
    #only(<part-b>)[
        - De cette comparaison découle un plan d'actions correctives à
            entreprendre pour faire converger les deux états
        - S'il n'y a pas d'écart entre l'état actuel et l'état désiré, aucune
            action n'est nécessaire
        - Ce plan d'action est ensuite exécuté sur la ressource gérée
        - Cette exécution modifie l'état réel de la ressource, ce qui amène la
            boucle à observer de nouveau cet état
        - Et comme ce cycle se répète à l'infini, tout écart ultérieur, qu'il
            soit dû à une panne ou à une modification externe, est
            automatiquement détecté et corrigé *|||*
    ]
]
#waypoint(<part-a>, advance: false)
- *Définition*: une boucle qui compare l'état actuel avec l'état désiré afin de
    les faire correspondre
#pause
#align(center, image(height: 80%, "/assets/image.png"))
#place(dx: 2cm, dy: -12cm, box(width: 100%, align(center, image(
    height: 90%,
    "/assets/image.png",
))))
#waypoint(<part-b>)

== Contrôleur et ressources
#speaker-note[
    - Début: 06:00 #h(5cm) Fin: *07:00*
    #only(<part-a>)[
        - Je vais aussi brièvement aborder deux autres concepts éssentiels que
            sont les ressources et les contrôleurs. *>>>*
        - Une ressource est simplement un objet qui représente l'état désiré, et
            un instantatné de l'état actuel
        - Chaque ressource est identifiée par un type, et éventuellement un nom
        - Le type permet de détermienr le schéma de donnée de cet état désiré et
            de l'éat actuel *>>>*
    ]
    #only(<part-b>)[
        - Quant au controleur, il s'agit s'implement de l'unité de code chargée
            d'implémenter la logique de réconciliation pour une ressource
            donnée.
        - Dans le système, les contrôleurs, et par extensions les ressources,
            sont regroupés en 3 domaines fonctionels: *>>>*
        - le domaine réseau, qui gère les routes, les addresses, le DHCP, etc.
            *>>>*
        - le domaine conteneur qui gère, non seulement les conteneurs a
            proprement dit, mais aussi les réseaux de conteneurs, les images,
            etc. *>>>*
    ]
    #only(<part-c>)[
        - et enfin le domaine système, qui gère les éléments qui n'ont pas été
            catégorisée dans les deux domaines *|||*
    ]
]

#waypoint(<part-a>, advance: false)
#pause
- *Ressource*: objet qui regroupe l'état désiré, et un instantané de l'état
    actuel
#waypoint(<part-b>)
- *Contrôleur*: implémente la _réconciliation_ pour une _ressource_ donnée
    #pause
    - Réseau
    #pause
    - Conteneur
    #waypoint(<part-c>)
    - Système

== Orchestration
#speaker-note[
    - Début: 07:00 #h(5cm) Fin: *08:30*
    #only(<part-a>)[
        - Enfin, je vous ai dit tantôt que la réconciliation est un processus
            qui se répète à l'infini.
        - Il faut donc décider qui est responsable de cette boucle. C'est ce que
            j'ai appelé l'orchestration
        - Et il y a deux modèles qui s'offrent à nous: le modèle centralisé et
            le modèle décentralisé *>>>*
    ]
    #only(<part-b>)[
        - Le modèle décentralisé est assez simple a expliquer:
        - Chaque ressource étant gérée par un contrôleur, le controleur décide
            comme il souhaite de l'implémentation de la boucle.
        - Par exemple une seul boucle pour toutes ses ressources, une boucle par
            type, une boucle par ressource, etc. *>>>*
        - Il a l'avantage d'être plus flexible mais est plus compliqué à
            implémenter.
        - C'est d'ailleur le modèle d'orchestration adopté par Kubernetes *>>>*
    ]
    #only(<part-c>)[
        - Le modèle centralisé est l'exacte opposé; dans ce modèle, un seul
            composant implémente la boucle, et va dispatcher la réconciliation
            vers le bon contrôleur.
        - Et il possède donc les avantages et inconvéients inverses: *>>>*
        - Il est plus rigide mais plus simple a implémenter.
        - Et c'est cette implémentation que j'ai retenu *>>>*
    ]
    #only(<part-d>)[
        - J'ai schématisé ici le fonctionnement général
        - Vous avez une boucle, implémenté dans un composant central qu'on verra
            plus tard
        - Et cette boucle itère sur chaque ressource, une à une, à l'infini
        - En réalité, il y a aussi un délai et une file d'attente *|||*
    ]
]

#waypoint(<part-a>, advance: false)
#waypoint(<part-b>)
- Décentralisé: chaque contrôleur/ressource à sa propre boucle
    #pause
    - Plus flexible mais plus compliqué à implémenter
#waypoint(<part-c>)
- *Centralisé*: une boucle centrale qui contrôle tout
    #pause
    - Plus rigide mais plus simple à implémenter
#waypoint(<part-d>)
#align(center, image("/assets/image-1.png"))


= Implémentation
#speaker-note[
    - Début: 08:40 #h(5cm) Fin: *08:50*
    - Maintenant que j'ai expliqué les concepts éssentiels, je vais vous parler
        de l'implémentation
    - En premier lieu, je vais vous donner une vue d'ensemble des technologies
        utilisée
]

== Vue d'ensemble
#speaker-note[
    - Début: 08:50 #h(5cm) Fin: *10:00*
    #only(<part-a>)[
        - Tout d'abord la solution ne se base sur aucune distribution ou
            programme existant:
            - pas de Debian, Ubuntu, etc.
            - et pas non plus de systemd ou autre *>>>*
        - Tout a donc été développé de zéro *>>>*
        - Et pour ça j'ai choisis de le faire en Rust
        - En ce qui concerne ce choix, il n'y a pas d'impératif particulier qui
            m'ont forcer à prendre Rust; il fallait simpement un language de bas
            niveau et Rust s'avère être celui avec lequel je suis le plus à
            l'aise
            *>>>*
    ]
    #only(<part-b>)[
        - Il y a deux composants principaux que je n'ai pas redéveloppé: *>>>*
        - Tout d'abord le runtime de conteneur, pour lequel j'ai choisi Podman
            - Ce choix a été fait durant le projet de semestre ou je l'ai
                comparé a d'autres solutions tel que containerd ou Docker, et
                Podman était la solution la plus simple a mettre en place dans
                le projet sans l'allourdir *>>>*
        - Pour le bootloader, le composant qui charge l'OS, j'ai choisi Limine.
            Là encore, aucun impératif particulier. Je souhaitais simplement
            découvrire un autre outil que GRUB et celui-ci remplissait
            l'ensemble des besoins *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Basé sur aucune distribution/programme existant
    #pause
    - Tout est développé de zéro
#pause
- Language de programmation: Rust
#waypoint(<part-b>)
- Composants externes:
    #pause
    - Runtime de conteneur: Podman
    #pause
    - Bootloader: Limine

== Immuabilité
#speaker-note[
    - Début: 10:00 #h(5cm) Fin: *12:00*
    #only(<part-a>)[
        - Un autre point important de l'implémentation c'est l'immuabilité du
            système de fichier.
        - En effet, plus le système démarre dans un état connu, plus il est
            simple de l'administrer, et pour cela, j'ai choisi de rendre le
            système de ficheir racine complètemetn immuable en utilisant
            SquashFs.
        - Ainsi, à chaque redémarrage, on connait exactement le contenu.
        - Maintenant je vous l'accorde, ne rien pouvoir écrire sur le système de
            fichier n'est pas très pratique. *>>>*
    ]
    #only(<part-b>)[
        - Par exemple, si vous souhaitez changer la configuration DNS comme on
            l'a fait dans la démo, cela doit être écrit dans le répertoire /etc.
        - Et on ne peut naturelemtn pas inclure ce répertoire dans l'image
            SquashFS, sinon on ne pourrait pas le changer. *>>>*
        - Pour ce faire, j'ai superposer un système temporaire par dessus
            l'image SquashFs en utilisant OverlayFS. *>>>*
        - OverlayFS c'est sans doutes quelque chose que vous avez déjà recontré
            si vous utilisez des conteneurs puisce que c'est comme ça que la
            pluspart fonctionnent *>>>*
    ]
    #only(<part-c>)[
        - Vous avez un ou plusieurs layer en lecture seule, ici notre système de
            ficheir racine, puis un dernier layer en écriture.
        - Vu que notre layer en écrutre est temporaire, càd en RAM, a chaque
            redémarrage, on perd tout
        - Très pratique car on revient a un état connu, mais assez embetant si
            vous devez tout reconfigurer a chaque fois. *>>>*
        - Pour ce faire, deux répertoire sont montés de manière persistente:
        - `/config` qui contient la dernière config valide que vous avez envoyé
        - et `/var` qui contient les données téléchargée tel que les images ou
            volumes de conteneurs *>>>*
    ]
    #only(<part-d>)[
        - Tout le reste des dossier et fichiers, conmme `/etc` peut être
            reconstruit grâce a cela *>>>*
        - A noter aussi que c'est a *vous* de choisir si vous souhaitez
            persister ces deux répertoires. SI vous les omettez de la config,
            vous obtenez un système complètemetn éhpmère *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Racine immuable #waypoint(<part-b>) #pause avec couche d'écriture temporaire
    #pause
    - SquashFs, Tmpfs, OverlayFs
#waypoint(<part-c>)
#pause
- `/config` et `/var` persistés
#waypoint(<part-d>)
- `/etc` et autres reconstruits à chaque redémarrage
#pause
- Persistance optionnelle: système peut être rendu entièrement éphémère

== Processus
#speaker-note[
    - Début: 12:00 #h(5cm) Fin: *13:00*
    #only(<part-a>)[
        - Enfin, en ce qui concerne la légèreté du système, cela est obtenu en
            minimsant le nombre de processus en cours d'exécution *>>>*
        - Dans le mode de fonctionnemetn normal, càd hors installation, je vous
            montre ici l'ensemble des processus en cours d'exécution.
        - Et j'insite là dessus, il n'y en a pas un seul de plus
        - ça veut aussi dire que je peux rapidemnt vous faire le tour de ces
            composants.
    ]
    #only(<part-b>)[
        - En premier lieu vous avez l'init, qui est chargé de mettre en place ce
            fameux système de fichier immuable que je vous ai décrit juste avant
        - Puis vous avez le supervisuer. Son rôle est simplemetn de démarrer les
            composants principaux dans le bon ordre:
        - C'est a dire démarrer tous les contrôleur que je vous ai décrit
            précédement
        - Puis une fois qu'ils sont tous prêt, démarrer le state-manager qui est
            le composant dans lequel la fameuse boucle d'orchestration et l'API
            sont implémenté.
    ]
    #only(<part-c>)[
        - Et puis chauqe contrôleur dispose de ses propre sous-processus, comme
            par exemple le runtime de conteneur
    ]
]

#waypoint(<part-a>, advance: false)
#pause
#align(center, image("/assets/image-2.png"))
#waypoint(<part-b>)
#waypoint(<part-c>)

== Pipeline de build
#speaker-note[
    - Début: 13:00 #h(5cm) Fin: *14:15*
    #only(<part-a>)[
        - Enfin je vais vous parler du système de build.
        - La particularité de ce projet c'est qu'il faut compiler non seulement
            une multitude d'application (les controleurs, l'init, etc.) *>>>*
        - Mais aussi le noyau Linux, puis packager tout ça sous différent
            format, comme des images SquashFS, eux-même assmeblé au final dans
            une image ISO *>>>*
        - En outre, je vise plusieurs architectures: x64 et ARM *>>>*
    ]
    #only(<part-b>)[
        - Pour ce faire, j'ai choisi Nix.
        - Je tiens ici a souligner que je parle de Nix en tant que système de
            build ou gestionnaire de paquet, mais pas de NixOS. CE sont deux
            choses étroitement liée mais indépendantes. *>>>*
        - L'avantage de Nix c'est que cela donne des builds 100% reproducible.
            C'est a dire que le build sur ma machine et le build sur un serveur
            de CI/CD donnera exactement le même résultat, à l'octet près.
        - En outre, c'est assez simple de disposer du même environement de dev
            *>>>*
    ]
    #only(<part-c>)[
        - L'inconvénient majeur c'est la lourdeur de Nix. Bien qu'il y ai un
            système de cache, dnas le cadre de Rust, l'unitée qui est mise en
            cache est un programme entier. Donc si vous modifier ne serait-ce
            qu'une ligne, tout va être recompilé. COmbiné au Rust, ça peut faire
            des temps de build assez long. *|||*
    ]
]
#waypoint(<part-a>, advance: false)
- Compilation de multiples composants : noyau Linux, contrôleurs, init...
#pause
- Packaging en images SquashFS, assemblées en image ISO
#pause
- Cible plusieurs architectures : x64 et ARM
#waypoint(<part-b>)
- Système de build: Nix
    #pause
    - Builds 100% reproductibles
#waypoint(<part-c>)
- Inconvénient: recompilations longues avec Rust

= Test & validation
#speaker-note[
    - Début: 14:15 #h(5cm) Fin: *14:30*
    - Bon, les conceptes et l'implémentation c'est bien pratique, mais dans le
        cas d'espèce, si ça ne fonctionne pas correctement, on ne vas pas aller
        très loin
    - Dans cette sectin je vais donc vous présenter brièvement comment est-ce
        que la solution est testée et surtout quels sont les chiffres qu'on peut
        en tirer.
]

== Stratégie de tests
#speaker-note[
    - Début: 14:30 #h(5cm) Fin: *15:00*
    #only(<part-a>)[
        - En ce qui concerne les tests unitaires et les tests d'intégration,
            c'est a dire lorsqu'un seul composant est testé, en isolation, il y
            en a une quarantaine *>>>*
        - Ce qui est plsu intéerssant, c'est les 14 tests de bout en bout. *>>>*
        - Ces tests éxecutent une VM entière, avec un disque neuf, et toutes les
            interactinos sont faite uniquement via l'API, de sorte a simuler un
            utilisateur réel *>>>*
    ]
    #only(<part-b>)[
        - À savoir aussi que tous ces tests sont effectués à chaque push pour
            s'assurer que ce que je merge dans main soit corect *>>>*
        - Par ailleur, ces tests servent de base a l'analyse de performance
            *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- 40 tests unitaires et intégrations
#pause
- 14 tests de bout en bout (E2E)
    #pause
    - VM isolée
    - Interaction uniquement via le client d'API
#waypoint(<part-b>)
- CI/CD sur chaque push bloquant le merge

== Performances
#speaker-note[
    - Début: 15:00 #h(5cm) Fin: *16:00*
    #only(<part-a>)[
        - Selon moi, l'aspect le plus intéressant de ce projet c'est les
            performnacnes que j'en tire
        - Sur 100 échantillons, et en suivant les même protocoles que les tests
            E2E pour être au plus proche d'un usage réel, *>>>*
        - Le système ne consomme que 160 méga de RAM pour télécharger et
            exécuter un conteneur. Et si je ne mesure que l'OS en tant que tel,
            sans le conteneur, ce chiffre tombe en dessosu des 80 méga. *>>>*
    ]
    #only(<part-b>)[
        - De même pour la vitesse, le démarrage du système puis d'un conteneur
            "à chaud", et par là j'entend lorsque l'image est déjà téléchargé,
            ne prend que 2.1 secondes
        - Si il est nécessaire de télécharger l'image, et bien cela prend au
            total 4.4s
        - Et enfin, le cycle complèt du démarrage de l'isntaller jusqu'a
            l'installation, le redémarrage, et le téléchargement du conteneur et
            son exécution prend 19.3s *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Sur 100 échantillons
- Même environnement que les tests unitaires
#pause
- RAM: *160 MiB* pour un conteneur, *\<80 MiB* pour le système seul
    - 160 MiB majoritairement dus à Podman
#waypoint(<part-b>)
- Rapidité:
    - 2.1 "hot start"
    - 4.4s "cold start"
    - 19.3s installation

// == Performances
// #image("/assets/image-5.png")

= Comparaison avec d'autres solutions
// == Critères
// #item-by-item[
//     - Automatisation: aucune action requise hormis insertion de l'ISO et *une
//         seule* commande
//     - Mémoire: en tout temps, moins de 300 MiB
//     - Rapidité: temps entre le démarrage de la VM, et le démarrage du conteneur
//     - Simplicité: aussi peu d'abstractions que possible
// ]

== Solutions étudiées
#speaker-note[
    - Début: 16:00 #h(5cm) Fin: *17:00*
]

=== Talos Linux
- *Orienté Kubernetes*
- Déclaratif et piloté par API
- Minimaliste

#pause
=== NixOS
- *Générique*
- se base sur Nix
- Déclaratif, mais pas en continu
- Complexe à prendre en main

== Synthèse
#speaker-note[
    - Début: 17:00 #h(5cm) Fin: *18:30*
]

#{
    show table.cell.where(y: 0): set text(weight: "bold")

    let mkcell(fill: none, default: none) = {
        return (..args) => {
            if args.pos().len() == 0 {
                table.cell(fill: fill, default)
            } else {
                table.cell(fill: fill, args.pos().at(0))
            }
        }
    }
    let y = mkcell(fill: green.transparentize(70%), default: sym.checkmark)
    let n = mkcell(fill: red.transparentize(70%), default: sym.crossmark)
    let o = mkcell(fill: gray.transparentize(70%), default: sym.nothing)
    let w = mkcell(fill: orange.transparentize(70%), default: sym.star)
    // @typstyle off
    table(
        columns: (auto, 1fr, 1fr, 1fr),
        table.header(
        [Critères],                         [ContainerOS], [NixOS],    [Talos]),
        [Automatisation],                   y(),           w(),        y(),
        [Mémoire requise en exécution],     y[*160 MiB*],  y[276 MiB], n[1.4 GiB],
        [Mémoire requise à l'installation], y[*160 MiB*],  n[762 MiB], n[1.4 GiB],
        [Rapidité d'installation],          [*36s*],       [300s],     [210s],
        [Rapidité de démarrage],            [*5.6s*],      [31s],      [65s],
        [Simplicité],                       y(),           y(),        n(),
    )
}

= Conclusion
== Rappel
#item-by-item[
    - *But*: un OS pour déployer des conteneurs de manière simple
    - *Solution*: basé sur rien, piloté par API et 100% déclaratif
    - *Résultats*: rapide et très léger (160 MiB)
]

== Perspectives
#item-by-item[
    - Customisation du noyau Linux
    - Plus de support (VPN, backups, etc.)
    - Extensions/plugins
        - Machines virtuelles
    - Job scheduling
]

= <touying:skip>
== Conclusion
#item-by-item[
    - Tous les objectifs de l'énoncé ont été atteints
    - Très satisfait des performances
    - Très intéressant
        - Mise en pratique des technologies vues en cours
        - Domaine du développement jamais touché
    - Projet personnel, amené à être maintenu dans le futur
]

= Questions

#filler-slide[]


