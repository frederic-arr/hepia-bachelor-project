#import "/lib/templates/single-page-common/lib.typ"

#show: lib.single-page.with(
    title: [
        OS pour le déploiement de services conteneurisés \
        #text(size: 0.86em, [Orientation: Informatique logicielle])
    ],
    header: text(font: "Liberation Sans")[
        Printemps 2026 \
        Session de bachelor
    ],
)
#set par(first-line-indent: 0em)

#text(font: "Liberation Sans")[*Descriptif:*] \
Le déploiement d'applications conteneurisées s'appuie désormais largement sur
des approches déclaratives, en particulier dans les environnements clusterisés.
En revanche, la configuration du système hôte reste souvent prise en charge par
des outils impératifs distincts, ce qui introduit une rupture entre la gestion
de l'infrastructure sous-jacente et celle des applications.

Cette séparation limite l'automatisation complète du déploiement et complique la
reproductibilité, en particulier dans les contextes de déploiement unitaire, où
les solutions conçues pour des environnements clusterisés peuvent s'avérer peu
adaptées.

Le but de ce projet est donc de palier à ce manque, en créant une distribution
Linux légère destinée à la mise en place d'infrastructures conteneurisées. Ce
système d'exploitation devra être léger en ressources, configurable via une API
entièrement déclarative et sécurisé (conteneurs rootless, etc.).
#v(3em)

#text(font: "Liberation Sans")[*Travail demandé:*] \
- Système développé ne se basant pas sur une distribution Linux existante.
- Gestion de la configuration du système réalisée de manière déclarative.
- Gestion des ressources nécessaires à la conteneurisation (conteneurs,
    interfaces réseau Ethernet) via ces fichiers déclaratifs.
- Exécution des conteneurs de manière rootless afin d'augmenter la sécurité.
- Administration du système uniquement via une API, sans accès distant (ssh) ou
    interface procédurale.
- Test et validation de la solution réalisée sur des use-cases réels.
- Comparaison de la solution développée à Talos Linux et NixOS.
- Création d'un système permettant de facilement déployer/installer la solution
    sur machine physique ou infrastructure Cloud.
