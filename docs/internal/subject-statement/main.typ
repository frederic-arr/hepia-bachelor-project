#import "/lib/templates/subject-statement/lib.typ": *

#set text(size: 12pt, lang: "fr", font: "Liberation Serif")
#show smallcaps: set text(font: "Alegreya Sans SC")

#subject-statement(
    header: text(font: "Liberation Sans")[
        Printemps 2026 \
        Session de bachelor
    ],
    title: [OS pour le déploiement de services conteneurisés],
    program: [Orientation: Informatique logicielle],
    author: (
        statement: [Candidat],
        name: [ARROYO Frédéric],
    ),
    field-of-study: (
        statement: [Filière d'études],
        name: [ISC],
    ),
    supervisors: (
        statement: [Professeur responsable],
        names: [GLÜCK Florent],
    ),
    client: none,
    internship: (
        statement: [Travail de bachelor soumis à une convention de stage en
            entreprise],
        value: [non],
    ),
    confidentiality-agreement: (
        statement: [Travail soumis à un contrat de confidentialité],
        value: [non],
    ),
)[
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
