use dioxus::prelude::*;

#[component]
pub fn Info() -> Element {
    rsx! {
        svg {
            class: "h-6 w-6",
            height: "24",
            width: "24",
            fill: "none",
            "stroke-linejoin": "round",
            stroke: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            "stroke-linecap": "round",
            "stroke-width": "2",
            "viewBox": "0 0 24 24",
            class: "icon icon-tabler icons-tabler-outline icon-tabler-info-circle",
            path { stroke: "none", fill: "none", d: "M0 0h24v24H0z" }
            path { d: "M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" }
            path { d: "M12 9h.01" }
            path { d: "M11 12h1v4h1" }
        }
    }
}

#[component]
pub fn ListDetail() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            height: "24",
            "viewBox": "0 0 24 24",
            width: "24",
            stroke: "currentColor",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "stroke-width": "2",
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            path { stroke: "none", d: "M0 0h24v24H0z", fill: "none" }
            path { d: "M13 5h8" }
            path { d: "M13 9h5" }
            path { d: "M13 15h8" }
            path { d: "M13 19h5" }
            path { d: "M3 4m0 1a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z" }
            path { d: "M3 14m0 1a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1z" }
        }
    }
}

pub fn Clipboard() -> Element {
    rsx! {
        svg {
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",
            height: "24",
            "viewBox": "0 0 24 24",
            "stroke-linecap": "round",
            stroke: "currentColor",
            width: "24",
            "stroke-width": "2",
            "stroke-linejoin": "round",
            class: "w-6 h-6",
            path { fill: "none", stroke: "none", d: "M0 0h24v24H0z" }
            path { d: "M9 5h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2h-2" }
            path { d: "M9 3m0 2a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v0a2 2 0 0 1 -2 2h-2a2 2 0 0 1 -2 -2z" }
            path { d: "M9 12h6" }
            path { d: "M9 16h6" }
        }
    }
}

#[component]
pub fn InstanceSelect() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            "stroke-linejoin": "round",
            "viewBox": "0 0 24 24",
            "stroke-width": "2",
            fill: "none",
            width: "24",
            "stroke-linecap": "round",
            stroke: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            height: "24",
            path { stroke: "none", d: "M0 0h24v24H0z", fill: "none" }
            path { d: "M3 5m0 2a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z" }
            path { d: "M6 8h.01" }
            path { d: "M9 8h.01" }
        }
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            width: "24",
            "stroke-width": "2",
            fill: "none",
            "viewBox": "0 0 24 24",
            stroke: "currentColor",
            "stroke-linejoin": "round",
            "stroke-linecap": "round",
            height: "24",
            xmlns: "http://www.w3.org/2000/svg",
            path { fill: "none", d: "M0 0h24v24H0z", stroke: "none" }
            path { d: "M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065z" }
            path { d: "M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" }
        }
    }
}

#[component]
pub fn DashedPlus() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            fill: "none",
            height: "24",
            stroke: "currentColor",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            stroke_width: "2",
            view_box: "0 0 24 24",
            width: "24",
            xmlns: "http://www.w3.org/2000/svg",
            path { d: "M0 0h24v24H0z", fill: "none", stroke: "none" }
            path { d: "M8.56 3.69a9 9 0 0 0 -2.92 1.95" }
            path { d: "M3.69 8.56a9 9 0 0 0 -.69 3.44" }
            path { d: "M3.69 15.44a9 9 0 0 0 1.95 2.92" }
            path { d: "M8.56 20.31a9 9 0 0 0 3.44 .69" }
            path { d: "M15.44 20.31a9 9 0 0 0 2.92 -1.95" }
            path { d: "M20.31 15.44a9 9 0 0 0 .69 -3.44" }
            path { d: "M20.31 8.56a9 9 0 0 0 -1.95 -2.92" }
            path { d: "M15.44 3.69a9 9 0 0 0 -3.44 -.69" }
            path { d: "M9 12h6" }
            path { d: "M12 9v6" }
        }
    }
}

pub fn DashedX() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            fill: "none",
            height: "24",
            stroke: "currentColor",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            stroke_width: "2",
            view_box: "0 0 24 24",
            width: "24",
            xmlns: "http://www.w3.org/2000/svg",
            path { d: "M0 0h24v24H0z", fill: "none", stroke: "none" }
            path { d: "M8.56 3.69a9 9 0 0 0 -2.92 1.95" }
            path { d: "M3.69 8.56a9 9 0 0 0 -.69 3.44" }
            path { d: "M3.69 15.44a9 9 0 0 0 1.95 2.92" }
            path { d: "M8.56 20.31a9 9 0 0 0 3.44 .69" }
            path { d: "M15.44 20.31a9 9 0 0 0 2.92 -1.95" }
            path { d: "M20.31 15.44a9 9 0 0 0 .69 -3.44" }
            path { d: "M20.31 8.56a9 9 0 0 0 -1.95 -2.92" }
            path { d: "M15.44 3.69a9 9 0 0 0 -3.44 -.69" }
            path { d: "M14 14l-4 -4" }
            path { d: "M10 14l4 -4" }
        }
    }
}
