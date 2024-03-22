module.exports = {
    mode: "all",
    content: [
        // include all rust, html and css files in the src directory
        "./src/**/*.{rs,html,css}",
        // include all html files in the output (dist) directory
        "./dist/**/*.html",
    ],
    theme: {
        extend: {},
        container: {
            center: true,
        }
    },
    plugins: [require("daisyui")],
    daisyui: {
        themes: [
            // "light",
            // "dark",
            "cupcake",
            // "bumblebee",
            // "emerald",
            // "corporate",
            // "synthwave",
            // "retro",
            // "cyberpunk",
            // "valentine",
            // "halloween",
            // "garden",
            // "forest",
            // "aqua",
            // "lofi",
            // "pastel",
            // "fantasy",
            // "wireframe",
            // "black",
            // "luxury",
            // "dracula",
            // "cmyk",
            // "autumn",
            // "business",
            // "acid",
            // "lemonade",
            // "night",
            // "coffee",
            // "winter",
            // "dim",
            // "nord",
            // "sunset",
        ],
    },
}