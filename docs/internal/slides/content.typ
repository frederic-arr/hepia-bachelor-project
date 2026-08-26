#import "/packages.typ": *
#import packages.touying: *
#import themes.metropolis: *

// Filler slides are to be used as the slide before the presentation starts
// or after the presentation ends. The projector should be frozen on those so
// that the presenter can do his configuration in the background.
#let filler-slide() = focus-slide[
    #image("/lib/assets/hepia-logo.svg")
]

#let cntr = counter("touying-slide-counter")

#filler-slide()

// Début: 00:00
// Fin: 00:20
// Contenu: Bonjour, aujourd'hui j'ai l'immense plaisir de vous présenter mon projet de
// Bachelor, intitulé "OS pour le déploiement de services conteneurisés".
// Cette présentation est scindé en deux parties: je vais tout d'abord vous
// introduire à la problématique et à la solution proposée du point de vue d
// l'utilisateur final de la solution, puis dans un second temps, ...
#title-slide()

= Introduction <touying:skip>
== Contexte
// Transition: Ce projet est un peu particulier car il s'agit tout d'abord d'un
// projet personnel, basé sur mon expérience dans l'administration et l'opération
// d'infrastructures modestes: typiquement pour de l'hébergement à titre personnel ou pour des petits projet avec quelques centaines d'utilisateur
// Dans ce contexte là
#speaker-note[
    - Début: 00:20
    - Fin: *00:55*
]

#item-by-item[
    - Déploiement modestes
    - Basé sur les conteneurs, mais sans clustering
    - Volonté d'automatiser les choses
]

== Problématique
- OS orientés conteneurs sont soit:
    - orienté Kubernetes (trop lourd)
    - orienté cloud (difficile à adapter sur environement embarqués)
- OS déclaratifs actuel sont complexe
#pause
#v(0.5cm)
#box(
    fill: rgb("e2001a").lighten(75%),
    inset: (top: 1cm, rest: 7.5mm),
    radius: 0.5em,
)[
    #place(dx: -7.5mm, dy: -1.5cm, box(
        fill: rgb("e2001a"),
        inset: 2.5mm,
        radius: 0.25em,
        strong(text(fill: white)[Question?]),
    ))

    Pourquoi n'existe-t-il pas un OS rudimentaire pour deployer des conteneurs
    en restant léger et simple?
]

== Solution proposée
- Distribution Linux spécialisée uniquement pour les conteneurs
- Configuration entièrement *déclarative*: _pas d'SSH_, pas de commandes
- Surface *minimale*: le stricte nécessaire pour les conteneurs
- Cible: déploiement modestes (VPS, embarqué, etc.)

= Démonstration <touying:skip>

= Suite de la présentation <touying:skip>
+ *Conception*: vue d'ensemble, architecture, composants principaux
+ *Implémentation*: Choix technique, fonctionnement
+ *Tests & Validation*: Comment s'assurer que la solution fonctionne bien?
+ *Comparaison avec d'autres solutions*: Est-ce que la solution développé est
    plus pratique que les alternatives?
+ *Discussion*: Quels sont ses limitations?

= Conception <touying:skip>

== Vue d'ensemble
#align(center, image("/assets/image-4.png"))

== Architecture
#align(center, image("/assets/image-2.png"))

= Implémentation <touying:skip>
== Choix techniques
- Language de programmation: Rust
- Environement de build: Nix
- Composants externes:
    - Runtime de conteneur: Podman
    - Bootloader: Limine

== Réconciliation
#align(center, image("/assets/image.png"))

= Test & Validation <touying:skip>
== Stratégie de tests
- 40 tests unitaires et intégrations
- 14 tests de bout en bout (E2E)
    - VM isolée
    - Interaction uniquement via le client d'API
- CI/CD sur chaque push bloquant le merge

== Performances
- Sur 100 échantillons
- Même environemetn que les tests unitaires
- RAM: *160 MiB* pour un conteneur, *\<80 MiB* pour le système seul
    - 160 MiB majoritairement dû à Podman
- Rapidité:
    - 2.1 "hot start"
    - 4.4s "cold start"
    - 19.3s installation

=== Temps jusqu'au démarrage du conteneur
#image("/assets/image-5.png")

= Comparaison avec d'autre solutions <touying:skip>
== Critères
- Automatisation: aucune action requise hormis insertion de l'ISO et *une seule*
    commande
- Mémoire: en tous temps, moins de 300 MiB
- Rapidité: temps entre le démarrage de la VM, et le démarrage du conteneur
- Simplicité: aussi peu d'abstraction que possible

== Solutions étudiées
=== Talos Linux
- *Orienté Kubernetes*
- OS déclaratif, minimal et piloté par API

=== NixOS
- *se base sur Nix*
- Déclaratif mais pas en continu

== Synthèse
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

= Discussion <touying:skip>
- Choix techniques justifiés


// = Introduction <touying:skip>

// == Contexte & Problématique

// - *Contexte*: Projet basé sur une expérience personelle
// #pause
// - *Constat*: Déployement typique pour projets modestes:
//     + Installer Debian/Ubuntu
//     + Configurer le système (réseau, SSH, paquets)
//     + Installer le runtime de conteneur
//     + Déployer des conteneurs
// #pause
// #v(0.5cm)
// #box(
//     fill: rgb("e2001a").lighten(75%),
//     inset: (top: 1cm, rest: 7.5mm),
//     radius: 0.5em,
// )[
//     #place(dx: -7.5mm, dy: -1.5cm, box(
//         fill: rgb("e2001a"),
//         inset: 2.5mm,
//         radius: 0.25em,
//         strong(text(fill: white)[Question?]),
//     ))

//     Pourquoi répéter toutes ces étapes lorsque la finalité exacte de l'OS est
//     connue?
// ]

// == Objectifs
// - *Fonctionnels*:
//     - Un OS prêt à l'emploi
//     - Configuration homogène et centralisée
//     - *Pas d'accès impératif*: tout passe par un fichier de configuration
// - *Techniques*:
//     - basé sur aucune distribution
//     - pas d'SSH

// = Démonstration

// == Suite de la présentation

// = Conceptes éssentiels
// == Ressources
// == Déclarativité
// #image("/assets/image.png")

// == Contrôleur
// - *Rôle*: implémente la logique déclarative pour une ressource
// - *Liste*:
//     - Réseau (`network-controller`)
//     - Conteneurs (`container-controller`)
//     - Système (`system-controller`): fichiers `/etc`

// == Boucle de contrôle
// #image("/assets/image-1.png")

// == Synthèse
// #grid(
//     columns: 2,
//     [
//         + Ressource stoquée
//         + Configuration de la ressource
//         + Envoyé au contrôleur
//         + Réconcilie la configuration avec la
//         + Ressource physique
//         + Renvoie le nouvel état pour le sauvegarder
//     ],
//     image("/assets/image-3.png"),
// )

// = Implémentation
// == Technologies
// == Composants
// #image("/assets/image-2.png")

// == Pipeline de build

// = Test & validation
// == Stratégie de test
// == Objectifs
// == Performances

// = Comparaison avec l'existant

// = Résultats

// = Discussions

// = Conclusion
// == Rappel
// - *Problématique*: déployer des conteneurs de manière simple, légère et rapide
// - *Solution*: OS piloté par API avec un fonctionnement déclaratif
// - *Technologies*:
//     - imlémentation _from scratch_, basé uniquement sur le noyau Linux
//     - développé en Rust
//     - Nix comme système de build

// == ...
// - *Validation*:
//     - Plusieurs tests unitaires
//     - Divers scénarios E2E testé dans la CI/CD
// - *Performances*:
//     - RAM: 160 MiB
//     - 2.3s démarrage "à chaud"
//     - 19.4s installation complète
// - *Raspberry Pi*: testé et 100% fonctionnel sur un Raspberry Pi 1B (2006)


// // == Contexte
// // #speaker-note[
// //     - Début: 00:55
// //     - Fin: *01:50*
// //     - Infrastructures modernes se ressemblent
// //     - Conteneurisation => réduit besoin spécifique
// //     - Retrouve même étapes => réseau, stockage, accès, plateforme de conteneurs
// //     - Donc tendance à la standardisation
// //     - *Et justement, c'est cette standardisation croissante qui fait apparaître
// //         le problème.*
// // ]

// // #item-by-item[
// //     - Ressemblances entre les infrastructures
// //     - La conteneurisation réduit les besoins spécifiques
// //     - Configuration récurrente:
// //         + Réseau
// //         + Stockage
// //         + Accès
// //         + Plateforme de conteneurs
// //     - Systèmes d'exploitation et outils encore généralistes
// // ]

// = Questions

// #filler-slide()
