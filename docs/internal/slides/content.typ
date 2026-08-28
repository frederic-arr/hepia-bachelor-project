/*
- Parler de l'IA
- Plus parler du projet de semestre
	- Quel est le but du projet de semesetre
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
// that the presenter can do his configuration in the background.
#let filler-slide() = focus-slide[
    #image("/lib/assets/hepia-logo.svg")
]

#let cntr = counter("touying-slide-counter")

#filler-slide()

#title-slide()

= Introduction <touying:skip>
== Contexte
#speaker-note[
    - Début: 00:30
    - Fin: *02:20*
    - *Ce projet n'est pas anodin, car c'est un projet que j'ai amené moi-même
        sur la table*
    - Modeste => pas de redondance, administré par une seul personne, sans
        clustering, etc.
    - Conteneurisation => s'affranchir de la complexité de l'installation et
        déploiement d'app
    - *Quelle forme prend la solution?*
]

- Projet personnel / basé sur mon expérience
#pause
- Déploiements modestes
#pause
- Basé sur les conteneurs
#pause
- Étapes communes
    + Installer l'OS
    #pause
    + Configurer le système (SSH, réseau, paquets)
    #pause
    + Installer le runtime de conteneur
    #pause
    + Déployer les conteneurs
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
        strong(text(fill: white)[Constat]),
    ))

    Malgré ces besoins simples, aucun OS ne remplit ce rôle simplement.
]

== Solution proposée
#speaker-note[
    - Début: 02:20
    - Fin: *04:20*
]
#grid(
    columns: 2,
    item-by-item[
        - Distribution Linux spécialisée
        - Surface *minimale*: le strict nécessaire pour les conteneurs
        - Pas d'SSH, de shell, ou de commandes
        - Piloté par API
        - Fichier de configuration unique
        - Système déclaratif
        - Homogène: bare-metal, VPS, embarqué, etc.
    ],
    [
        #pause
        #align(center, image("/assets/image-4.png")),
    ],
)


= Démonstration <touying:skip>

= Suite de la présentation <touying:skip>
#speaker-note[
    - Début: 04:20
    - Fin: *05:00*
]

+ *Conception*
+ *Implémentation*
+ *Tests & Validation*
+ *Comparaison avec d'autres solutions*

= Conception

== Réconciliation
#speaker-note[
    - Début: 05:00
    - Fin: *07:00*
    - Pour un objet donné
]

- *Définition*: faire converger un état désiré avec l'état actuel
#pause
#align(center, image(height: 90%, "/assets/image.png"))

== Contrôleur et ressources
#speaker-note[
    - Début: 07:00
    - Fin: *09:00*
]
- *Ressource*: objet qui regroupe l'état désiré, et un instantané de l'état
    actuel
    #pause
    - avec un type, un nom, un schéma
#pause
- *Contrôleur*: implémente la _réconciliation_ pour une _ressource_ donnée
    - Réseau
    - Conteneur
    - Système

== Orchestration
#speaker-note[
    - Début: 09:00
    - Fin: *10:00*
]

- *Centralisée* vs Décentralisé
    - Centralisé: une boucle centrale qui contrôle tout
    - Décentralisé: chaque contrôleur/ressource à sa propre boucle
#pause
#align(center, image("/assets/image-1.png"))


= Implémentation
== Vue d'ensemble
#item-by-item[
    - Basé sur aucune distribution existante
    - Language de programmation: Rust
    - Composants externes:
        - Runtime de conteneur: Podman
        - Bootloader: Limine
]

== Immuabilité
#item-by-item[
    - Racine immuable avec couche d'écriture temporaire
        - SquashFs, Tmpfs, OverlayFs
    - Uniquement `/var` est persisté
    - `/etc` et autres dossiers reconstruits à chaque redémarrage
]

== Processus
#align(center, image("/assets/image-2.png"))

== Pipeline de build
#item-by-item[
    - Système de build: Nix
    - Build 100% reproducible
    - Tout est compilé (noyau, composants internes et externes)
    - Fournis aussi un environnement de dév.
]

= Test & validation
== Stratégie de tests
#item-by-item[
    - 40 tests unitaires et intégrations
    - 14 tests de bout en bout (E2E)
        - VM isolée
        - Interaction uniquement via le client d'API
    - CI/CD sur chaque push bloquant le merge
]

== Performances
#item-by-item[
    - Sur 100 échantillons
    - Même environnement que les tests unitaires
    - RAM: *160 MiB* pour un conteneur, *\<80 MiB* pour le système seul
        - 160 MiB majoritairement dus à Podman
    - Rapidité:
        - 2.1 "hot start"
        - 4.4s "cold start"
        - 19.3s installation
]

== Performances
#image("/assets/image-5.png")

= Comparaison avec d'autres solutions
== Critères
#item-by-item[
    - Automatisation: aucune action requise hormis insertion de l'ISO et *une
        seule* commande
    - Mémoire: en tout temps, moins de 300 MiB
    - Rapidité: temps entre le démarrage de la VM, et le démarrage du conteneur
    - Simplicité: aussi peu d'abstractions que possible
]

== Solutions étudiées
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

==
#item-by-item[
    - Tous les objectifs de l'énoncé ont été atteints
    - Très satisfait des performances
    - Très intéressant
        - Mise en pratique des technologies vues en cours
        - Domaine du développement jamais touché
    - Projet personnel, amené à être maintenu dans le futur
]

= Questions

#filler-slide()
