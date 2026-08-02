#let full-ref(label, supplement: none) = {
    show ref: it => {
        let el = it.element

        if el.func() == heading {
            return link(el.location(), [#it,~#emph(el.body)])
        } else if el.func() == figure {
            return link(el.location(), [#it,~#emph(el.caption.body)])
        } else {
            return highlight(fill: red, text(color: black)[*INVALID*])
        }
    }

    state("use-short-caption", false).update(_ => true)
    ref(label, supplement: supplement)
    state("use-short-caption", false).update(_ => false)
}

#let num-ref(label, supplement: none) = {
    ref(label, supplement: supplement)
}

#let nota-bene(body) = {
    text(style: "italic")[N.B.: #body]
}

#let chapter-num-ref(label) = {
    num-ref(label, supplement: "chapitre")
}

#let chapter-full-ref(label) = {
    full-ref(label, supplement: "chapitre")
}

#let appendix-full-ref(label) = {
    full-ref(label, supplement: "annexe")
}

#let chapters-full-ref(..labels) = [
    chapitres #(
        labels.pos().map(label => full-ref(label)).join(", ", last: " et ")
    )
]

#let figure-full-ref(label) = {
    full-ref(label, supplement: "figure")
}

#let figure-num-ref(label) = {
    num-ref(label, supplement: "figure")
}

#let table-full-ref(label) = {
    full-ref(label, supplement: "tableau")
}

#let table-num-ref(label) = {
    num-ref(label, supplement: "tableau")
}

#let code-num-ref(label) = {
    num-ref(label, supplement: "code")
}

#let appendix-num-ref(label) = {
    num-ref(label, supplement: "annexe")
}
