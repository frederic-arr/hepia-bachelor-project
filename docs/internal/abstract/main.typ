#import "/lib/templates/single-page-common/lib.typ"

#show: lib.single-page.with(
    anchor: "Résumé",
    title: [Résumé],
)

La conteneurisation s'est imposée comme le mode standard de déploiement des
applications, avec un mode d'opération de plus en plus déclaratif. Les systèmes
d'exploitation sous-jacents reposent toutefois majoritairement sur des
distributions génériques, dont le modèle d'exécution demeure impératif et qui
n'intègrent pas nativement la conteneurisation. Les solutions plus spécialisées
existantes présentent des lacunes majeures dans le contexte de déploiements
modestes ou embarqués. Ce travail présente la conception et l'implémentation
d'un système d'exploitation minimaliste, dédié au déploiement déclaratif de
conteneurs sur une machine unique. Le système repose sur un modèle de ressource
homogène, décrivant conjointement la configuration de l'hôte et des conteneurs,
dont l'état est maintenu par un orchestrateur centralisé au moyen d'une boucle
de réconciliation continue. L'implémentation, réalisée en Rust sans se baser sur
une distribution existante, comprend un processus d'initialisation, un
environnement utilisateur restreint dépourvu de shell et d'accès interactif,
ainsi qu'un ensemble de contrôleurs dédiés au réseau, aux conteneurs et à la
configuration système. La validation du système, menée au moyen de tests
unitaires, d'intégration et de bout en bout, démontre sa capacité à déployer une
application complexe et à en préserver l'état à travers un redémarrage. Les
mesures effectuées établissent un démarrage de conteneur en 5.1 secondes depuis
une image ISO, sans installation préalable, ainsi qu'une exécution possible d'un
serveur web avec seulement 160 MiB de mémoire vive, un seuil nettement inférieur
à celui des solutions comparées. Ce travail conclut sur l'atteinte des objectifs
techniques fixés, tout en identifiant des limites, notamment l'absence de
journalisation et de supervision, ouvrant la voie à des développements futurs.

#v(1cm)
#align(center, image(
    height: 5cm,
    fit: "contain",
    "../lib/assets/containeros.jpg",
))
