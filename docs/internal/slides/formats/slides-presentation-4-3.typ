#import "/packages.typ": *
#import packages.touying: config-common
#import "../config.typ"

#config.mk-slides(
    ratio: "4-3",
    config: config-common(
        handout: false,
    ),
)
