#let full-ref(label, supplement: true) = {
    show ref: it => {
        let el = it.element
        if type(el) != type(heading([])) { return it }

        link(el.location(), [#it,~#emph(el.body)])
    }

    ref(label)
}

#let num-ref(label, supplement: true) = {
    show ref: it => {
        let el = it.element
        if type(el) != type(heading([])) { return it }

        link(el.location(), [#it,~#emph(el.body)])
    }

    ref(label, supplement: none)
}

#let named-ref(label) = {
    show ref: it => {
        let el = it.element
        if el.func() != heading { return it }

        link(el.location(), [#el.body])
    }

    ref(label)
}

#let nota-bene(body) = {
    text(style: "italic")[N.B.: #body]
}
