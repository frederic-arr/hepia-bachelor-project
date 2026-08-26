#import "/packages.typ": *
#import packages.touying: *
#import themes.metropolis: *

#let mk-slides(ratio: "16-9", config: none) = {
    set text(lang: "fr", font: "Liberation Sans")
    show: metropolis-theme.with(
        aspect-ratio: "16-9",
        font: "Liberation Sans",
        config,
        config-info(
            title: [OS pour le déploiement de services conteneurisés],
            subtitle: [Défense du projet de bachelor],
            author: [Frédéric ARROYO],
            date: datetime(year: 2026, month: 09, day: 01),
            institution: [HEPIA \/\/ HES-SO Genève],
            logo: image("/lib/assets/hepia-logo.svg"),
        ),
        header: [
            #text(
                white,
                weight: "bold",
                utils.display-current-heading(level: 2),
            )
            #h(1fr)
            #text(white, size: 0.75em, [#utils.display-current-heading(
                level: 1,
            )])
        ],
        config-colors(
            primary: rgb("#e2001a"),
            primary-light: rgb("#d6c6b7"),
            secondary: rgb("#e2001a"), // Top bar
            neutral-lightest: rgb("#ffffff"), // Slides background
            neutral-dark: rgb("#bababa"),
            // neutral-darkest: rgb("#ff00ff"),
        ),
    )

    include "content.typ"
}
