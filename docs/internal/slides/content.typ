/*
- p12. légende schémaa
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
Bonjour, aujourd'hui, j'ai l'immense plaisir de vous présenter mon travail de
bachelor, qui s'intitule OS pour le déploiement de services conteneurisés.

Tous d'abord, je vais vous présenter la problématique et le contexte de se travail,
ainsi que vous présenter la solution développée avant de passer à la suite.
*/

= Introduction <touying:skip>
== Problématique
#speaker-note[
    - Début: 00:20 #h(5cm) Fin: *01:50*
    #only(<part-a>)[
        - *Ce projet s'intéresse l'administration d'environnement modeste*
        - Par modestes, il faut entendre des environnements administrés par une
            seule personne, sans clustering, sans déploiement multirégion, et
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
#pause
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
            qui me tient à cœur: je voulais avant tout voir mes compétences à
            l'œuvre. *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Projet de semestre, octobre #sym.arrow.double mars)
    #pause
    - Identification de solution existante adéquate: *aucune*
    #pause
    - Conceptualisation de très haut niveau
    #pause
    - Recherche des "briques" logicielles
#waypoint(<part-b>)
- IA: tentative pour débogage + amélioration CI/CD
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
    - Et je vais donc maintenant vous présenter les concepts essentiels de cette
        solution, à commencer par la réconciliation. *|||*
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
        - Je vais aussi brièvement aborder deux autres concepts essentiels que
            sont les ressources et les contrôleurs. *>>>*
        - Une ressource est simplement un objet qui représente l'état désiré, et
            un instantané de l'état actuel
        - Chaque ressource est identifiée par un type, et éventuellement un nom
        - Le type permet de détermienr le schéma de donnée de cet état désiré et
            de l'éat actuel *>>>*
    ]
    #only(<part-b>)[
        - Quant au contrôleur, il s'agit simplement de l'unité de code chargée
            d'implémenter la logique de réconciliation pour une ressource
            donnée.
        - Dans le système, les contrôleurs, et par extension les ressources,
            sont regroupés en 3 domaines fonctionnels: *>>>*
        - le domaine réseau, qui gère les routes, les adresses, le DHCP, etc.
            *>>>*
        - le domaine conteneur qui gère, non seulement les conteneurs a
            proprement dit, mais aussi les réseaux de conteneurs, les images,
            etc. *>>>*
    ]
    #only(<part-c>)[
        - et enfin le domaine système, qui gère les éléments qui n'ont pas été
            catégorisés dans les deux domaines *|||*
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
    - Début: 07:00 #h(5cm) Fin: *08:00*
    #only(<part-a>)[
        - Enfin, je vous ai dit tantôt que la réconciliation est un processus
            qui se répète à l'infini.
        - Il faut donc décider qui est responsable de cette boucle. C'est ce que
            j'ai appelé l'orchestration.
        - Et il y a deux modèles qui s'offrent à nous: le modèle centralisé et
            le modèle décentralisé *>>>*
    ]
    #only(<part-b>)[
        - Le modèle décentralisé est assez simple a expliquer:
        - Chaque ressource étant gérée par un contrôleur, le contrôleur décide
            comme il souhaite de l'implémentation de la boucle.
        - Par exemple, une seule boucle pour toutes ses ressources, une boucle
            par type, une boucle par ressource, etc. *>>>*
        - Il a l'avantage d'être plus flexible, mais est plus compliqué à
            implémenter.
        - C'est d'ailleurs le modèle d'orchestration adopté par Kubernetes *>>>*
    ]
    #only(<part-c>)[
        - Le modèle centralisé est l'exact opposé; dans ce modèle, un seul
            composant implémente la boucle, et va dispatcher la réconciliation
            vers le bon contrôleur.
        - Et il possède donc les avantages et inconvénients inverses: *>>>*
        - Il est plus rigide, mais plus simple à implémenter.
        - Et c'est cette implémentation que j'ai retenue *>>>*
    ]
    #only(<part-d>)[
        - J'ai schématisé ici le fonctionnement général
        - Vous avez une boucle, implémentée dans un composant central qu'on
            verra plus tard
        - Et cette boucle itère sur chaque ressource, une à une, à l'infini
        - En réalité, il y a aussi un délai et une file d'attente *|||*
    ]
]

#waypoint(<part-a>, advance: false)
#waypoint(<part-b>)
- Décentralisé: chaque contrôleur/ressource à sa propre boucle
    #pause
    - Plus flexible, mais plus compliqué à implémenter
#waypoint(<part-c>)
- *Centralisé*: une boucle centrale qui contrôle tout
    #pause
    - Plus rigide, mais plus simple à implémenter
#waypoint(<part-d>)
#align(center, image("/assets/image-1.png"))


= Implémentation
#speaker-note[
    - Début: 08:00 #h(5cm) Fin: *08:10*
    - Maintenant que j'ai expliqué les concepts essentiels, je vais vous parler
        de l'implémentation
    - En premier lieu, je vais vous donner une vue d'ensemble des technologies
        utilisées
]

== Vue d'ensemble
#speaker-note[
    - Début: 08:10 #h(5cm) Fin: *09:10*
    #only(<part-a>)[
        - Tout d'abord la solution ne se base sur aucune distribution ou
            programme existant: pas de Debian, Ubuntu, et pas non plus de
            systemd ou autre *>>>*
        - Tout a donc été développé de zéro *>>>*
        - Et pour ça j'ai choisi de le faire en Rust
        - En ce qui concerne ce choix, il n'y a pas d'impératif particulier qui
            m'a forcé à prendre Rust; il fallait simplement un langage de bas
            niveau et Rust s'avère être celui avec lequel je suis le plus à
            l'aise *>>>*
        - Il y a deux composants principaux que je n'ai pas redéveloppés *>>>*
    ]
    #only(<part-b>)[
        - Tout d'abord, le runtime de conteneur, pour lequel j'ai choisi Podman
            - Ce choix a été fait durant le projet de semestre ou je l'ai
                comparé a d'autres solutions, telles que containerd ou Docker,
                et Podman était la solution la plus simple a mettre en place
                dans le projet sans l'alourdir *>>>*
        - Pour le bootloader, le composant qui charge l'OS, j'ai choisi Limine.
            Là encore, aucun impératif particulier. Je souhaitais simplement
            découvrir un autre outil que GRUB et celui-ci remplissait l'ensemble
            des besoins *|||*
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
    - Début: 09:10 #h(5cm) Fin: *11:10*
    #only(<part-a>)[
        - Un autre point important de l'implémentation, c'est l'immuabilité du
            système de fichier.
        - En effet, plus le système démarre dans un état connu, plus il est
            simple de l'administrer, et pour cela, j'ai choisi de rendre le
            système de fichier racine complètement immuable en utilisant
            SquashFs.
        - Ainsi, à chaque redémarrage, on connait exactement le contenu.
        - Maintenant je vous l'accorde, ne rien pouvoir écrire sur le système de
            fichier n'est pas très pratique. *>>>*
    ]
    #only(<part-b>)[
        - Par exemple, si vous souhaitez changer la configuration DNS comme on
            l'a fait dans la démo, cela doit être écrit dans le répertoire /etc.
        - Et on ne peut naturellement pas inclure ce répertoire dans l'image
            SquashFS, sinon on ne pourrait pas le changer. *>>>*
        - Pour ce faire, j'ai superposé un système temporaire par-dessus l'image
            SquashFs en utilisant OverlayFS. *>>>*
        - OverlayFS c'est sans doute quelque chose que vous avez déjà rencontré
            si vous utilisez des conteneurs, puisque c'est comme ça que la
            plupart fonctionnent *>>>*
    ]
    #only(<part-c>)[
        - Vous avez un ou plusieurs layer en lecture seule, ici notre système de
            fichier racine, puis un dernier layer en écriture.
        - Vu que notre layer en écriture est temporaire, càd en RAM, a chaque
            redémarrage, on perd tout
        - Très pratique, car on revient à un état connu, mais assez embêtant si
            vous devez tout reconfigurer a chaque fois. *>>>*
        - Pour ce faire, deux répertoires sont montés sur des partitions ext4:
        - `/config` qui contient la dernière config valide que vous avez envoyée
        - et `/var` qui contient les données téléchargées, telles que les images
            ou volumes de conteneurs *>>>*
    ]
    #only(<part-d>)[
        - Tout le reste des dossiers et fichiers, comme `/etc` peut être
            reconstruit grâce a cela *>>>*
        - A noter aussi que c'est a *vous* de choisir si vous souhaitez
            persister ces deux répertoires. SI vous les omettez de la config,
            vous obtenez un système complètement éphémère *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Racine immuable #waypoint(<part-b>) #pause avec couche d'écriture temporaire
    #pause
    - SquashFs, Tmpfs, OverlayFs
#waypoint(<part-c>)
#pause
- `/config` et `/var` persistés sur partitions ext4
#waypoint(<part-d>)
- `/etc` et autres reconstruits à chaque redémarrage
#pause
- Persistance optionnelle: système peut être rendu entièrement éphémère

== Processus
#speaker-note[
    - Début: 11:10 #h(5cm) Fin: *12:10*
    #only(<part-a>)[
        - Enfin, en ce qui concerne la légèreté du système, cela est obtenu en
            minimisant le nombre de processus en cours d'exécution *>>>*
        - Dans le mode de fonctionnement normal, càd hors installation, je vous
            montre ici l'ensemble des processus en cours d'exécution.
        - Et j'insiste là-dessus, il n'y en a pas un seul de plus
        - ça veut aussi dire que je peux rapidement vous faire le tour de ces
            composants.
    ]
    #only(<part-b>)[
        - En premier lieu vous avez l'init, qui est chargé de mettre en place ce
            fameux système de fichier immuable que je vous ai décrit juste avant
        - Puis vous avez le superviseur. Son rôle est simplement de démarrer les
            composants principaux dans le bon ordre:
        - C'est-à-dire démarrer tous les contrôleurs que je vous ai décrits
            précédemment
        - Puis, une fois qu'ils sont tous prêts, démarrer le state-manager qui
            est le composant dans lequel la fameuse boucle d'orchestration et
            l'API sont implémentés.
    ]
    #only(<part-c>)[
        - Et puis chaque contrôleur dispose de ses propres sous-processus, comme
            par exemple, le runtime de conteneur
    ]
]

#waypoint(<part-a>, advance: false)
#pause
#align(center, image("/assets/image-2.png"))
#waypoint(<part-b>)
#waypoint(<part-c>)

== Pipeline de build
#speaker-note[
    - Début: 12:10 #h(5cm) Fin: *13:10*
    #only(<part-a>)[
        - Enfin je vais vous parler du système de build.
        - La particularité de ce projet c'est qu'il faut compiler non seulement
            une multitude d'applications (les contrôleurs, l'init, etc.) *>>>*
        - Mais aussi le noyau Linux, puis packager tout ça sous différent
            format, comme des images SquashFS, eux-mêmes assemblé au final dans
            une image ISO *>>>*
        - En outre, je vise plusieurs architectures: x64 et ARM *>>>*
    ]
    #only(<part-b>)[
        - Pour ce faire, j'ai choisi Nix.
        - Je tiens ici à souligner que je parle de Nix en tant que système de
            build ou gestionnaire de paquet, mais pas de NixOS. CE sont deux
            choses étroitement liées, mais indépendantes. *>>>*
        - L'avantage de Nix c'est que cela donne des builds 100% reproductibles.
            C'est-à-dire que le build sur ma machine et le build sur un serveur
            de CI/CD donneront exactement le même résultat, à l'octet près.
        - En outre, c'est assez simple de disposer du même environnement de dev
            *>>>*
    ]
    #only(<part-c>)[
        - L'inconvénient majeur, c'est la lourdeur de Nix. Bien qu'il y ait un
            système de cache, dans le cadre de Rust, l'unité qui est mise en
            cache est un programme entier. Donc, si vous modifier ne serait-ce
            qu'une ligne, tout va être recompilé. Combiner au Rust, ça peut
            faire des temps de build assez long. *|||*
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
    - Début: 13:10 #h(5cm) Fin: *13:20*
    - Bon, les concepts et l'implémentation, c'est bien pratique, mais dans le
        cas d'espèce, si ça ne fonctionne pas correctement, on ne va pas aller
        très loin
    - Dans cette section, je vais donc vous présenter brièvement comment la
        solution est testée et surtout quels sont les chiffres qu'on peut en
        tirer.
]

== Stratégie de tests
#speaker-note[
    - Début: 13:20 #h(5cm) Fin: *13:50*
    #only(<part-a>)[
        - En ce qui concerne les tests unitaires et les tests d'intégration,
            c'est-à-dire lorsqu'un seul composant est testé, en isolation, il y
            en a une quarantaine *>>>*
        - Ce qui est plus intéressant, c'est les 14 tests de bout en bout. *>>>*
        - Ces tests exécutent une VM entière, avec un disque neuf, et toutes les
            interactions sont faite uniquement via l'API, de sorte a simuler un
            utilisateur réel *>>>*
    ]
    #only(<part-b>)[
        - À savoir aussi que tous ces tests sont effectués à chaque push pour
            s'assurer que ce que je merge dans main soit correct *>>>*
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
    - Début: 13:50 #h(5cm) Fin: *14:50*
    #only(<part-a>)[
        - En ce qui concerne les performances, cela a été mesuré sur 100
            échantillons, et en suivant les mêmes protocoles que les tests E2E
            pour être au plus proche d'un usage réel, *>>>*
        - Le système ne consomme que 160 mégas de RAM pour télécharger et
            exécuter un conteneur. Et si je ne mesure que l'OS en tant que tel,
            en omettant Podman, ce chiffre tombe en dessous des 80 mégas. *>>>*
    ]
    #only(<part-b>)[
        - De même pour la vitesse, le démarrage du système, puis d'un conteneur
            "à chaud", et, par là, j'entends lorsque l'image est déjà
            téléchargée, ne prend que 2.1 secondes
        - S'il est nécessaire de télécharger l'image, et bien cela prend au
            total 4.4s
        - Et enfin, le cycle complet du démarrage de l'installer jusqu'a
            l'installation, le redémarrage, et le téléchargement du conteneur et
            son exécution prend 19.3s *|||*
    ]
]

#waypoint(<part-a>, advance: false)
- Sur 100 échantillons
- Même environnement que les tests E2E
#pause
- RAM: *160 MiB* pour un conteneur, *\<80 MiB* pour le système seul
    - 160 MiB majoritairement dus à Podman
#waypoint(<part-b>)
- Rapidité:
    - 2.1 "hot start"
    - 19.3s installation

= Comparaison avec d'autres solutions
#speaker-note[
    - Début: 14:50 #h(5cm) Fin: *15:00*
    - Enfin, je vais comparer ce qui a été développé a deux autres solutions:
        Talos Linux et NixOS *>>>*
]

== Solutions étudiées
#speaker-note[
    - Début: 15:00 #h(5cm) Fin: *16:00*
    #only(<part-a>)[
        - Ces deux solutions sont les solutions identifiées comme les plus
            proches des besoins lors du projet de semestre. *>>>*
        - La première, Talos Linux, est une distribution orientée Kubernetes:
            elle est déclarative et pilotée par API, mais son minimalisme reste
            "contraint" par les besoins de Kubernetes et vous le verrez, cela se
            répercute sur les chiffres. *>>>*
    ]
    #only(<part-b>)[
        - La seconde est NixOS. C'est une solution générique, se basant sur Nix,
            qu'on a déjà vu tout à l'heure.
        - Et l'aspect particulier de NixOS c'est que vous "buildez" votre OS:
            quand vous faites un changement, c'est un nouveau build que vous
            appliquez à la machine
        - Et c'est de là que vient l'aspect déclaratif, mais pas continu: il est
            possible de faire des changements hors du cadre du système de build
        - Enfin, NixOS ce n'est pas la solution la plus facile à prendre en main
    ]
]

#waypoint(<part-a>, advance: false)
#pause
=== Talos Linux
- *Orienté Kubernetes*
- Déclaratif et piloté par API
- Minimaliste

#waypoint(<part-b>)
=== NixOS
- *Générique*
- se base sur Nix
- Déclaratif, mais pas en continu
- Complexe à prendre en main

== Synthèse
#speaker-note[
    - Début: 16:00 #h(5cm) Fin: *18:30*
    #only(<part-a>)[
        - Ce tableau synthétise la comparaison sur six critères.
        - Pour chaque solution, j'ai utilisé la configuration par défaut et fait
            les changements minimums pour pouvoir y déployer un conteneur.
        - Aussi, ContainerOS correspond a la solution développée durant de
            travail, et les mesures de temps différent un petit peu de ce que je
            vous ai montré tout à l'heure, car cela a été fait sur un
            environnement différent.
    ]
    #only(<part-b>)[
        - Tout d'abord, l'automatisation, c'est le fait d'effectuer le moins
            d'actions possible sur l'ensemble du cycle de vie de la machine.
            Pour ça, ContainerOS et Talos, il ne suffit que de démarrer dans
            l'installeur et effectuer une seule commande, peu importe
            l'environnement
        - Ce n'est toutefois pas le cas de NixOS, où l'installation initiale
            nécessite plusieurs actions manuelles *>>>*
    ]
    #only(<part-c>)[
        - En ce qui concerne la mémoire, la limite acceptable est placée à 300
            MiB.
        - ContainerOS est nettemetn plus léger, aussi bien à l'exécution qu'à
            l'installation, avec 160 MiB durant les deux phases.
        - Pour NixOS, l'exécition est légère, mais l'installation nécessitant de
            build certains aspects, cela consomme beaucoup de RAM
        - Quand a Talos, le simple fait d'exécuter Kubernetes prend une quantité
            considérable de RAM.
    ]
    #only(<part-d>)[
        - Sur la rapidité, il n'y a pas de limites acceptables, mais vous pouvez
            voir que ContainerOS est clairement plus rapide que les autres
            solutions.
        - Pour Talos Linux, c'est toujours lié à Kubernetes
        - Et de même, pour NixOS, la lenteur à l'installation est toujours liée
            à cette notion de build. En revanche, la lenteur d'exécution est
            simplement due au fait que NixOS étant générique, la distribution
            démarre une multitude de services
    ]
    #only(<part-e>)[
        - Enfin, sur la simplicité, ContainerOS et NixOS se rejoignent grâce à
            leur fichier de configuration unique, alors que Talos reste plus
            complexe du fait de son orientation cluster.
        - Ce tableau confirme donc que pour le cas d'usage visé — un déploiement
            unitaire, non-cluster — ContainerOS répond de façon plus adéquate
            que les deux solutions existantes. *|||*
    ]
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
    waypoint(<part-a>, advance: false)
    table(
        columns: (auto, 1fr, 1fr, 1fr),
        table.header([Critères], [ContainerOS], [NixOS], [Talos]),
        waypoint(<part-b>), [Automatisation], y(), w(),
        y(), waypoint(<part-c>), [Mémoire requise en exécution], y[*160 MiB*],
        y[276 MiB],
        n[1.4 GiB],
        [Mémoire requise à l'installation],
        y[*160 MiB*],

        n[762 MiB], n[1.4 GiB], waypoint(<part-d>), [Rapidité d'installation],
        [*36s*], [300s], [210s], [Rapidité de démarrage],
        [*5.6s*], [31s], [65s], waypoint(<part-e>),
        [Simplicité], y(), y(), n(),
    )
}

= Conclusion
== Rétrospective
- *But*: un OS pour déployer des conteneurs de manière simple
#pause
- *Solution*: basé sur rien, piloté par API et 100% déclaratif
#pause
- *Résultats*:
    #pause
    - Rapide et très léger (160 MiB)
    #pause
    - Tous les objectifs ont été atteints
#pause
- *Difficultés*:
    - Bugs dans des composants externes (Podman, WSL, bibliothèque Rust)
    - Champ large
    - Évolution en terrain inconnu

== Perspectives
- *Court terme*: mise à jour des composants + observabilité
#pause
- *Moyen terme*: plus de fonctionnalité dans les contrôleurs existants
#pause
- *Long terme*: système de plugin

= <touying:skip>
== Conclusion
- Très intéressant
    - Mise en pratique des technologies vues en cours
    - Domaine du développement jamais touché
- Projet personnel, amené à être maintenu dans le futur

= Questions

#filler-slide[]


