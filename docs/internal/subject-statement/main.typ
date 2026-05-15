#import "/lib/templates/single-page-common/lib.typ"

#lib.single-page(
    title: [
        OS pour le déploiement de services conteneurisés \
        #text(size: 0.86em, [Orientation: Informatique logicielle])
    ],
    header: text(font: "Liberation Sans")[
        Printemps 2026 \
        Session de bachelor
    ],
)[
    #set par(first-line-indent: 0em)

    *Descriptif:*
    Le déploiement d'applications conteneurisées s'appuie désormais largement
    sur des approches déclaratives, en particulier dans les environnements
    clusterisés. En revanche, la configuration du système hôte reste souvent
    prise en charge par des outils impératifs distincts, ce qui introduit une
    rupture entre la gestion de l'infrastructure sous-jacente et celle des
    applications.

    Cette séparation limite l'automatisation complète du déploiement et
    complique la reproductibilité, en particulier dans les contextes de
    déploiement unitaire, où les solutions conçues pour des environnements
    clusterisés peuvent s'avérer peu adaptées.

    *Travail demandé:*
    - Sur la base du travail de semestre, implémenter une distribution Linux
        légère qui permet:
        - de gérer la configuration du système de manière déclarative à travers
            des fichiers YAML;
        - de gérer les ressources nécessaires à la conteneurisation (conteneurs,
            interfaces réseau Ethernet) via ces fichiers déclaratifs;
        - d'exécuter des conteneurs de manière rootless avec Podman;
        - d'administrer le système uniquement au travers d'une API gRPC (sans
            SSH ou interface procédurale), et
        - si le temps le permet, packager la solution sous forme d'un installeur
            ISO;
    - écrire un processus sommaire de test et de validation de la solution, et
    - comparer l'implémentation à Talos Linux et NixOS.
]
