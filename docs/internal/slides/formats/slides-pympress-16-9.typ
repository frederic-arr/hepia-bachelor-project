#import "/packages.typ": *
#import packages.touying: config-common
#import "../config.typ"

#config.mk-slides(
    ratio: "16-9",
    config: config-common(
        handout: false,
        show-notes-on-second-screen: right,
    ),
)
