#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <cicd>,
    caption: [Pipeline CI/CD],
    note: [
        En bleu, le pipeline pour la documentation (thèse, slides, etc.). En
        rouge, le pipeline pour le code Rust. Les deux pipelines s'exécutent de
        manière indépendante.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(
            label: <cicd-doc-check>,
            (0, 0),
            title: [Check],
            subtitle: [Formatting, spelling],
            stroke: blue,
        )
        node(
            label: <cicd-doc-build>,
            (1, 0),
            title: [Build],
            subtitle: [PDF for Thesis, Slides, ...],
            stroke: blue,
        )
        edge(<cicd-doc-check>, <cicd-doc-build>, "-|>")

        node(
            label: <cicd-code-proto>,
            (0, 1.5),
            title: [Proto],
        )
        node(
            label: <cicd-code-clippy>,
            (0, 2.5),
            title: [Build],
        )
        node(
            label: <cicd-code-check>,
            enclose: (<cicd-code-proto>, <cicd-code-clippy>),
            inset: 5mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Check*
            ])),
        )

        node(
            label: <cicd-code-doc>,
            (1.1, 1),
            title: [Doctest],
        )
        node(
            label: <cicd-code-unit>,
            (1.1, 2),
            title: [Unit Tests],
        )
        node(
            label: <cicd-code-integ>,
            (1.1, 3),
            title: [Integration Tests],
        )
        node(
            label: <cicd-code-test>,
            enclose: (<cicd-code-doc>, <cicd-code-unit>, <cicd-code-integ>),
            inset: 5mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Tests*
            ])),
        )
        edge(<cicd-code-check>, <cicd-code-test>, "-|>")

        node(
            label: <cicd-code-build>,
            (2.8, 2),
            title: [*Build ISO*],
            stroke: red,
        )
        edge(<cicd-code-test>, <cicd-code-build>, "-|>")

        node(
            label: <cicd-code-e2e>,
            (4, 2),
            title: [*E2E Tests*],
            stroke: red,
        )
        edge(<cicd-code-build>, <cicd-code-e2e>, "-|>")
    },
)
