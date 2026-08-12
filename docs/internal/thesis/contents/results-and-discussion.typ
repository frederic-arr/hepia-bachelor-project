#import "../lib.typ": *

= Résultats et Discussion <ch:results>
Ce chapitre dresse le bilan du travail réalisé. Les résultats obtenus sont
d'abord présentés, tant sur le plan fonctionnel qu'au regard des objectifs
académiques fixés en introduction, avant qu'un retour rétrospectif ne soit fait
sur les principaux choix techniques effectués. Les difficultés rencontrées au
cours du développement sont ensuite détaillées, avant que le chapitre ne se
conclue par les perspectives d'évolution envisageables pour le système.

== Résultats fonctionnels
Les scénarios de validation présentés au #chapter-full-ref(<ch:validation>)
démontrent que le système fonctionne bien dans trois configurations
représentatives, et la comparaison avec NixOS et Talos~Linux établit que le
système présente une empreinte mémoire et des performances nettement meilleures.

Le système fonctionne également sur un Raspberry Pi 1B, sans qu'aucune
adaptation notable n'ait été nécessaire pour ce matériel. Bien que le projet
visait à être déployé sur des Raspberry Pi modernes, la capacité du système à
fonctionner sur un modèle aussi ancien illustre concrètement l'atteinte de
l'objectif de légèreté, ce matériel disposant de ressources sensiblement
inférieures à celles des modèles visés initialement.

D'un point de vue fonctionnel, la valeur ajoutée du système réside dans sa
rapidité de démarrage et dans sa faible empreinte mémoire, deux propriétés
mesurées au #chapter-full-ref(<ch:validation:bench>) et confirmées par la
comparaison avec les solutions existantes#sym.space.narrow: le système démarre
un conteneur en quelques secondes et fonctionne avec une empreinte mémoire de
160 MiB, ce qui permet un déploiement quasi immédiat, aussi bien depuis une
image ISO que depuis une installation sur disque.

Le système présente toutefois des limites fonctionnelles. Deux fonctionnalités
identifiées lors de la planification ne sont pas implémentées#sym.space.narrow:
la gestion de tâches planifiées ("job scheduling") ainsi que le monitoring du
système. L'absence de tâches planifiées fait qu'il est plus compliqué d'exécuter
des routines régulières (par exemple des sauvegardes), tandis que l'absence de
monitoring complique la détection et le diagnostic d'une défaillance. Ces deux
limitations restent toutefois partiellement contournables dans l'état actuel du
système, ces deux fonctionnalités pouvant elles-mêmes être déployées sous forme
de conteneurs. En outre, seule une partie des options de configuration du réseau
et des conteneurs a été implémentée, principalement dans un souci de temps.

== Résultats académiques <results-academic>
L'ensemble des objectifs techniques formulés dans l'énoncé du sujet sont remplis
par le système développé. Les résultats fonctionnels présentés à le
#chapter-full-ref(<ch:validation>), en particulier la validation des trois
scénarios de bout en bout et les mesures effectuées confirme cela. Les deux
fonctionnalités non implémentées ne figurent pas parmi les objectifs centraux de
l'énoncé et sont écartées afin de concentrer l'effort sur la robustesse du
système, jugée prioritaire. Au regard de l'ensemble de ces éléments, les
objectifs de l'énoncé du sujet sont considérés comme atteints avec grande
satisfaction.

La planification initiale du travail de diplôme est établie sous la forme d'un
diagramme de Gantt, structuré par semaine sur l'ensemble de la durée du projet.
La #figure-num-ref(<plan-gantt>) présente cette planification, complétée a
posteriori par l'avancement effectif de chaque tâche.

#{
    set page(flipped: true)
    include "../diagrams/plan-gantt.typ"
}

La planification reste fidèle à l'avancement effectif jusqu'au début du mois de
juillet. À partir de cette période, la correspondance entre la planification et
l'avancement réel devient plus approximative. La tâche "Test et validation"
figurant sur le diagramme ne recouvre pas l'ensemble des activités de test
menées durant le projet, le système faisant l'objet de tests continus dès les
premières étapes de son développement, notamment au moyen des tests unitaires et
d'intégration décrits au #chapter-full-ref(<ch:validation>). Cette tâche est
avancée et prolongée au-delà de la durée prévue, recouvrant une partie de la
période initialement réservée à l'amélioration du code, cette dernière tâche
ayant elle-même été anticipée. Ce chevauchement s'explique par la nécessité de
consolider certains composants avant que les tests de bout en bout ne puissent
être exécutés de manière fiable.

== Difficultés rencontrées <ch:results:limits>
Le développement du système, bien qu'ayant abouti à une solution fonctionnelle,
conforme aux objectifs fixés, et satisfaisante, ne s'est pas déroulé sans
obstacle. Plusieurs difficultés techniques sont rencontrées durant le
développement du système. Certaines relèvent de particularités peu documentées
d'un composant, d'autres de bugs identifiés dans des dépendances externes, et
d'autres encore de contraintes propres à l'environnement de développement
utilisé.

Un comportement inattendu est observé lors de la mise en place de l'initrd
nécessaire au démarrage du système. Lorsque la mémoire disponible est
insuffisante, l'initrd n'est que partiellement chargé, ce qui introduit une
confusion importante lors du diagnostic#sym.space.narrow: le processus `/init`
s'exécute normalement et parvient à lire certains fichiers, tandis que d'autres
fichiers, censés être présents, sont rapportés comme inexistants. Ce mode de
défaillance silencieux est surprenant, d'autant plus qu'aucune information
affichée sur la console par le noyau ne laissait supposer un tel problème.

Un bug est également identifié dans smoltcp, seule bibliothèque Rust utilisable
comme client DHCP dans le contexte de ce système. La bibliothèque standard de
Rust fourni plusieurs types qui sont réutilisés à travers tout l'écosystème,
mais certaines, dont smoltcp, réimplémentent leurs propres types, notamment un
équivalent de `std::time::Instant`. Une interface `From` est alors fournie pour
convertir un type vers l'autre#sym.space.narrow; dans le cas de smoltcp, cette
conversion est incorrectement implémentée et retourne systématiquement l'instant
présent, faussant l'ensemble des calculs de délai reposant sur cette conversion
@bib-smoltcp-issue. Le bug c'est avérer trivialement réglable et, étant donné
qu'il s'agit d'un élément essentiel du système, et compte tenu de l'absence
d'autres alternatives, un correctif permettant de régler ce bogue a été crée,
soumis via une pull request, et accepté, sur le dépôt de smoltcp
@bib-smoltcp-pull.

Un bug est aussi rencontré dans Podman, version 5.8.4. Podman revendique une
compatibilité "drop-in" avec Docker, sans que cette compatibilité soit exacte en
pratique. La fonction `list_containers` retourne notamment une structure
légèrement différente#sym.space.narrow: Docker rapporte un état
`"status": "exited"` associé au champ `"stopped": true`, alors que Podman
rapporte, pour un état équivalent, la valeur `"stopped"` pour ce même champ
`status`. Ce bug est connu depuis 2023 @bib-podman-issue et n'est corrigé par le
projet qu'en mars 2026 @bib-podman-pull, la correction n'étant intégrée à une
version officielle qu'avec la publication de la version 6.0.0, le 24 juin 2026
@bib-podman-release. L'intégration de cette version dans Nix, celle-ci
constituant une "breaking release", nécessite un délai supplémentaire et demeure
bloquée~@bib-podman-nix. Le paquet Nix officiel est alors repris et adapté
directement à partir de son code source, l'aspect "breaking" de la version 6.0
de Podman n'ayant pas d'impact dans le cadre de ce système.

Au-delà de ce bug, Podman présente une autre particularité#sym.space.narrow: son
fonctionnement sans daemon ("daemon-less") ne s'applique qu'à l'utilisation de
la ligne de commande `podman`. Aucune interface native ("bindings") n'est
fournie pour une intégration directe. Podman doit donc être exécuté comme une
API, avec laquelle une interaction s'effectue via HTTP ou TCP, ce qui revient,
en pratique, à le traiter comme un daemon.

Un bug est par ailleurs observé sous WSL, environnement utilisé pour une partie
du développement. La création d'un fichier temporaire via `open2` et l'option
`O_TMPFILE`, suivie d'une tentative de le rendre permanent via `linkat`, produit
une erreur indiquant que le fichier n'existe pas. Ce bug affecte les tests
d'intégration du contrôleur système, responsable de la gestion des
fichiers#sym.space.narrow; ces tests échouent sous WSL, mais s'exécutent
correctement sur une machine Debian standard.

Le temps de build de Nix constitue une autre difficulté notable. Nix isole
intégralement chaque build et ne tire volontairement pas parti de la compilation
incrémentale, ce qui pénalise particulièrement Rust, dont les temps de
compilation sont réputés élevés. Un build complet de la partie userspace en Rust
peut atteindre 5 minutes, sans compter le temps de build du noyau, lui-même
considérable même sur une machine puissante. La bibliothèque communautaire
"crane" permet de mettre en cache une partie des dépendances externes, mais sa
documentation reste limitée et son usage impose une structuration précise du
projet.

== Rétrospective
Le choix de Rust et de Nix constitue, malgré les difficultés rencontrées, un
choix technique approprié. Le système présente une robustesse satisfaisante, au
sens où une erreur ne provoque pas d'arrêt inattendu du système, et la
reproductibilité des builds constitue un avantage important dans un contexte de
développement réparti sur plusieurs appareils et intégré à une pipeline CI/CD.

Le choix architectural d'un modèle centralisé est plus nuancé. Ce modèle
simplifie effectivement l'ordonnancement, objectif initial de ce choix, mais
réduit la flexibilité du système, désavantage identifié dès la phase de
conception. L'implémentation des différents contrôleurs révèle des besoins
particuliers à chacun, ce qui n'est pas imputable au modèle centralisé en tant
que tel, mais complexifie le composant central à mesure que les cas d'usage se
diversifient. Ce choix est considéré comme neutre au regard du compromis obtenu.

Le choix de gRPC pour la communication entre le contrôleur central et les
contrôleurs répond avant tout à un besoin de rapidité de mise en œuvre. Cette
communication reste, dans une certaine mesure, lourde à maintenir, bien que
cette lourdeur soit largement masquée pour l'utilisateur final. Un protocole
custom, basé sur des sockets UNIX, constituerait potentiellement une alternative
plus adaptée, mais le système actuel fonctionne de manière satisfaisante,
rendant cette amélioration secondaire.

== Perspectives
Le système répond, dans son périmètre actuel, aux objectifs fixés, mais son
architecture n'exclut pas des évolutions ultérieures. Plusieurs perspectives
d'évolution se dégagent de ce travail, tant dans le périmètre initial que dans
des directions plus larges.

Concernant l'usage primaire du système, centré sur les conteneurs, trois
fonctionnalités apparaissent prioritaires#sym.space.narrow: l'ajout d'une couche
d'observabilité, l'ajout d'un mécanisme de planification de tâches similaire à
CRON, et la gestion plus poussée du stockage. D'autres extensions sont
envisageables dans un second temps, notamment l'extension de la gestion du
réseau, par exemple pour le support des VPN ou du routage plus complexe, ainsi
que l'extension de la gestion des conteneurs avec un modèle similaire aux "Pods"
de Kubernetes ou de Podman.

Le projet peut également être rendu plus générique. La partie orchestration et
ordonnancement est conceptuellement découplée des ressources qu'elle gère, ce
qui ouvre la voie à une bibliothèque générique. La gestion du réseau, en
particulier, réimplémente en grande partie des fonctionnalités déjà couvertes
par systemd-networkd ou Netplan#sym.space.narrow; cette partie pourrait être
extraite sous forme de composant externe qui serait à la fois une alternative à
ces deux solutions, tout en étant une partie intégrale du système.

Le système peut aussi être étendu pour supporter la virtualisation via libvirt,
l'orchestrateur ne percevant, dans tous les cas, qu'une ressource à réconcilier,
le comportement spécifique restant délégué au contrôleur correspondant. Cela
nécessiterait toutefois d'implémenter divers aspects supplémentaires pour
permettre de déléguer un disque ou une carte graphique à une machine
virtuelle.Un autre axe d'amélioration consisterait à renforcer la légèreté du
système, en se basant sur une configuration du noyau minimale, dans laquelle
seuls certains pilotes seraient disponibles.

Ces perspectives illustrent qu'un nombre restreint de modifications
conceptuelles est peut transformer fondamentalement la portée du projet,
l'implémentation sous-jacente de ces modifications restant néanmoins nettement
plus complexe que leur description conceptuelle.
