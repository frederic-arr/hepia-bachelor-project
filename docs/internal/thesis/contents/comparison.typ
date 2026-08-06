#import "../lib.typ": *

= Comparaison avec les solutions existantes
Ce chapitre compare directement le système implémenté aux deux solutions les
plus proches identifiées durant le projet de semestre, à savoir NixOS et Talos
Linux. Les critères d'évaluation retenus sont d'abord définis, avant que la
méthodologie de mesure ne soit précisée et que chaque solution ne soit présentée
et évaluée individuellement selon ces critères; une synthèse comparative conclut
ce chapitre.

== Critères
Les critères utilisés sont repris du projet de semestre @bib-semester-projet.

=== Automatisation
Le cycle de vie de la solution doit être entièrement automatisable, dès la phase
d'installation. Plus spécifiquement, l'installation, la configuration initiale
et le déploiement de l'hôte doivent pouvoir être réalisés à distance, de manière
programmatique et sans intervention humaine. L'installation et la configuration
du système doivent en outre reposer sur les mêmes commandes, sans procédure
dédiée à la seule phase d'installation.

=== Légèreté
La solution doit pouvoir fonctionner avec une quantité de mémoire vive limitée,
l'objectif visé étant un fonctionnement stable avec 256 MiB de RAM, et la limite
acceptable étant fixée à 512 MiB. Le critère est considéré comme atteint
uniquement si la limite de 512 MiB est respectée à la fois lors de
l'installation et lors de l'exécution.

=== Rapidité
La solution doit permettre un déploiement rapide, aussi bien lors d'une
installation initiale que lors d'un démarrage ultérieur du système. Ce critère
est évalué au moyen de deux mesures: le temps d'installation, mesuré entre
l'entrée dans l'environnement d'installation et le démarrage du conteneur
configuré, et le temps de démarrage, mesuré entre le lancement de la machine
virtuelle et le démarrage de ce même conteneur.

=== Simplicité
La solution doit être aussi simple que possible à l'utilisation. La
configuration doit être centralisée, clairement structurée et facilement
modifiable par une personne ayant des connaissances dans le domaine concerné.

== Méthodologie
Pour chaque solution évaluée, la configuration retenue correspond à la
configuration par défaut recommandée par la documentation officielle du projet,
sans optimisation spécifique.

La rapidité est mesurée au moyen d'un conteneur effectuant une requête HTTP vers
l'hôte dès son démarrage, selon la même procédure que celle décrite au
#chapter-full-ref(<ch:validation:bench>): l'instant de réception de cette
requête par l'hôte marque la fin de la mesure, aussi bien pour le temps
d'installation que pour le temps de démarrage.

La légèreté est évaluée à partir de ce même protocole de mesure de la rapidité:
une machine virtuelle est démarrée avec une quantité de mémoire vive donnée,
puis la requête HTTP attendue est surveillée pendant une durée maximale de cinq
minutes. L'absence de réception de cette requête dans ce délai est interprétée
comme un échec du démarrage du conteneur pour la quantité de mémoire testée. En
cas d'échec, une nouvelle tentative sera effectuée avec plus de mémoire vive
jusqu'à ce que cela fonctionne.

== NixOS
NixOS est une distribution Linux généraliste construite autour du gestionnaire
de paquets Nix, dont le périmètre couvre l'ensemble des usages d'un système
d'exploitation traditionnel, la gestion de conteneurs n'en constituant qu'un
usage parmi d'autres. La configuration du système repose sur un langage
déclaratif et fonctionnel, permettant une forte reproductibilité des
installations ainsi que des mises à jour atomiques. L'installation d'un système
NixOS s'effectue traditionnellement à partir d'un support d'installation
exécutant un installeur interactif, la configuration déclarative n'intervenant
qu'une fois le système de base installé.

L'évaluation de la solution est réalisée sur la version 25.11, qui utilise la
version 6.12.62 du noyau Linux, publiée le 30 novembre 2025. Cette version est
disponible sous la licence MIT. La configuration utilisée pour cette évaluation
est disponible dans #repo("misc/nixos/vm.nix"). La mémoire requise s'élève à 276
MiB en exécution et à 762 MiB à l'installation. Le temps d'installation mesuré
est de 5 minutes, et le temps de démarrage de 31 secondes.

Le critère d'automatisation n'est que partiellement atteint: l'installation de
NixOS nécessite un installeur interactif ou un mécanisme d'initialisation
externe (par exemple nixos-anywhere @bib-nixos-anywhere) chargé d'effectuer
l'installation initiale, cette étape reposant ainsi sur des commandes distinctes
de celles utilisées pour la configuration déclarative subséquente du système. Le
critère de légèreté n'est par ailleurs pas atteint, la mémoire requise à
l'installation (762 MiB) dépassant la limite acceptable de 512 MiB, bien que la
mémoire requise en exécution seule (276 MiB) reste dans à cette limite.

== Talos Linux
Talos Linux est une distribution Linux immuable et minimale, développée par
Sidero Labs et conçue exclusivement pour l'exécution de clusters Kubernetes. Le
système de fichiers racine est en lecture seule et l'ensemble de
l'administration s'effectue via une API dédiée. La configuration du système est
décrite intégralement par un unique fichier déclaratif, appliqué dès le premier
démarrage de la machine.

L'évaluation porte sur la version 1.11.5, qui utilise la version 6.12.52 du
noyau Linux, publiée le 6 novembre 2025. Ce logiciel est distribué sous la
licence MPL-2.0. La configuration utilisée pour cette évaluation est disponible
dans #repo("misc/talos/config.yaml"). La mémoire requise s'élève à 1.4 GiB,
aussi bien à l'installation qu'en exécution. Le temps d'installation mesuré est
de 210 secondes, et le temps de démarrage de 65 secondes.

Le critère d'automatisation est atteint: la même configuration déclarative,
transmise via l'API du système, est utilisée aussi bien pour l'installation
initiale que pour toute reconfiguration ultérieure, sans qu'aucune commande
distincte ne soit requise entre ces deux phases. Le critère de légèreté n'est en
revanche pas atteint, la mémoire requise (1.4 GiB) dépassant très largement la
limite acceptable de 512 MiB, aussi bien à l'installation qu'en exécution.

== Synthèse
Le #table-num-ref(<fig-sota-comp>) rassemble, pour les trois systèmes évalués,
les résultats obtenus pour chacun des critères définis en début de chapitre:

#{
    set text(size: 10pt)
    show table.cell.where(y: 0): set text(weight: "bold")
    show table.cell.where(x: 0): set text(weight: "bold")
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
    figure(
        label: <fig-sota-comp>,
        caption: [Comparaison des solutions existantes],
        note: [
            #sym.checkmark signifie que le critère est atteint \
            #sym.crossmark signifie que le critère n'est pas atteint \
            #sym.star signifie que le critère n'est que partiellement atteint \
            #sym.nothing signifie que la valeur n'est pas applicable
        ],

        // @typstyle off
        table(
            columns: (auto, 1fr, 1fr, 1fr),
            table.header(
            [Critères],                         [ContainerOS], [NixOS],       [Talos]),
            [Dernière date],                    o(),           o[2025-11-30], o[2025-11-06],
            [Version évaluée],                  o(),           o[25.11],      o[1.11.5],
            [Version du noyau],                 o[6.18.11],    o[6.12.62],    o[6.12.52],
            [Licence],                          o(),           o[MIT],        o[MPL-2.0],
            table.hline(stroke: 2pt + black),
            [Automatisation],                   y(),           w(),           y(),
            [Mémoire requise en exécution],     y[*160 MiB*],  y[276 MiB],    n[1.4 GiB],
            [Mémoire requise à l'installation], y[*160 MiB*], n[762 MiB], n[1.4 GiB],
            [Rapidité d'installation],          [*36s*],      [300s],     [210s],
            [Rapidité de démarrage],            [*5.6s*],     [31s],      [65s],
            [Simplicité],                       y(),          y(),        n(),
        ),
    )
}

La solution développe, ContainerOS, est la seule des trois solutions à
satisfaire l'ensemble des critères retenus. NixOS échoue le critère de légèreté
à l'installation, sa consommation mémoire dépassant alors la limite acceptable
malgré une empreinte en exécution conforme, ainsi que le critère
d'automatisation, en raison de la dépendance à un installeur interactif ou à un
mécanisme d'initialisation externe distinct de la configuration déclarative
elle-même. Talos Linux échoue quant à lui le critère de légèreté de manière
constante, aussi bien à l'installation qu'en exécution, ainsi que le critère de
simplicité, son périmètre étant orienté vers la gestion de clusters Kubernetes
plutôt que vers le déploiement unitaire visé. Sur le plan de la légèreté,
ContainerOS requiert environ deux fois moins de mémoire que NixOS et près de dix
fois moins que Talos Linux, tout en restant dans les deux cas sous la limite
acceptable de 512 MiB, y compris sur une machine dotée de seulement 256 MiB de
RAM. Sur le plan de la rapidité, ContainerOS démarre environ cinq fois plus vite
que NixOS et près de dix fois plus vite que Talos Linux; le temps d'installation
est quant à lui inférieur d'un facteur proche de dix par rapport à ces deux
mêmes solutions.
