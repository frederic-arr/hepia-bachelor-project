#import "/packages.typ": *
#import packages.touying: *
#import themes.metropolis: *
#import "config.typ"

#config.mk-slides(
    ratio: "16-9",
    config: config-common(
        handout: true,
        new-section-slide-fn: new-section-slide.with(
            config: config-common(freeze-slide-counter: true),
        ),
    ),
), )
