#import "/packages.typ": *;
#import packages.codly: *
#import packages.codly-languages: *

#set text(lang: "fr", hyphenate: false)
#set par(justify: true)

#set page(numbering: "I")

// TOOD: Acknowledgments

#show: codly-init.with()
#codly(languages: codly-languages)

// TODO: AI usage declaration

// TODO: Table of contents
// TODO: Acronyms list
// TODO: Table of figures
// TODO: Table of appendices

/*
- Introduction
- Architecture générale

- "Démonstration"
- Test et validation
    - Scénarios de validation
    - Benchmarking
- Comparaison avec d'autre solutions
- Résultats (ACADÉM + FONCTIONEL: parler de la valeur ajoutée, des limitations, etc)
- Discussion (TECHNIQUE: parler des bon et mauvais choix, limitations, etc)
- Conclusion
*/

#outline()
#pagebreak()

#include "extra/acknowledgements.typ"
#include "/subject-statement/main.typ"
#include "/abstract/main.typ"

#include "contents/introduction.typ"

#set page(numbering: "1")
#counter(page).update(1)
#set heading(numbering: "1.")

#include "contents/overview.typ"
#include "contents/architecture.typ"
#include "contents/implementation.typ"
#include "contents/validation.typ"
#include "contents/comparison.typ"
#include "contents/results.typ"
#include "contents/discussion.typ"

#set heading(numbering: none)
#include "contents/conclusion.typ"

/*
= *NON CATÉGORISÉ*
- Parler des dépendances (crates)
- Coordination des resources
    - Décrire la gestion de l'état désiré/courrant
        - Scheduling (c.f. l'ADR 1)
    - gRPC
    - Décrire les API/services internes
- Parler de qualité de code (linting, CI/CD, etc)
- Parler de la sécurité

= Test et validation de la solution
== Scénarios de validation
- Quelques scénarios type.

== Benchmarking
- Divers mesures qu'on peut faire, e.g.:
    - empreinte mémoire; pour ceci, deux méthodes:
        - pour qqc de précis, utiliser l'allocateur Peak en Rust
        - de manière plus générale donner X MB de RAM à la VM, et chercher la
            plus grosse allocation qu'on puisse faire depuis un conteneur
    - time-to-boot
    - ???

== Comparaison à d'autres solutions
Comparer à Talos Linux et NixOS par rapport aux benchmarks et aux scénarios.

= Résultats
- Dire que la solution a été validé sur l'ensemble des objectifs techniques
    énoncés
- Dire que la solution a été validé par rapport à d'autres solutions
    "concurrentes" avec satisfaction
- Dire que, outre les points énoncés, la solution remplis les objectifs
    personnels
- Dire que les objectifs du TB on tous été atteint
- Parler de la valeur ajoutée d'un point de vue fonctionnel
- Parler (aussi!) des contraintes/points négatifs de la solution d'un point de
    vue fonctionnel

= Discussion
- Parler des bon et mauvais choix techniques
- Parler des dificutlé technique

= Perspectives
- Boite à idée

/*
- Preface
    - Conventions
    - Terms and definitions
- Introduction
    - Context
    - Problems
    - Objectives
    - Abstract
    - Structure
- *****
- Conclusion
- Annex
- Bibliography
*/

= Conclusion
= Annexe A
= Annexe B
= Glossaire
= Bibliographie
*/
