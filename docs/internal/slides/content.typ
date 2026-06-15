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
    - Début: 00:55
    - Fin: *01:50*
    - Infrastructures modernes se ressemblent
    - Conteneurisation => réduit besoin spécifique
    - Retrouve même étapes => réseau, stockage, accès, plateforme de conteneurs
    - Donc tendance à la standardisation
    - *Et justement, c'est cette standardisation croissante qui fait apparaître
        le problème.*
]

#item-by-item[
    - Ressemblances entre les infrastructures
    - La conteneurisation réduit les besoins spécifiques
    - Configuration récurrente:
        + Réseau
        + Stockage
        + Accès
        + Plateforme de conteneurs
    - Systèmes d'exploitation et outils encore généralistes
]

= Merci pour votre attention!

#filler-slide()
