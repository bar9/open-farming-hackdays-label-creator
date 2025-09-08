module.exports = {
    mode: "all",
    content: [
        // include all rust, html and css files in the src directory
        "./src/**/*.{rs,html,css}",
        // include all html files in the output (dist) directory
        "./dist/**/*.html",
    ],
    theme: {
        extend: {
            colors: {
                // BioSuisse/Knospe colors
                'knospe-green': '#39A22B',
                'knospe-red': '#D21E1F',
                'knospe-lightgreen': '#ACBB00',
                'knospe-darkgreen': '#00393F',
                'knospe-tablegreen': '#267B39',
                'knospe-darkred': '#8C0002',
                'knospe-darkblue': '#383384',
                'knospe-promo': '#DEE555',
                // Biomondo colors
                'bio-seagreen': '#267b39',
                'bio-frostee': '#cdefd5',
                'bio-ottoman': '#ecf9ef',
                'bio-gray': '#737373'
            },
            fontFamily: {
                'futura': ['Futura', 'Arial', 'sans-serif'],
                'opensans': ['Open Sans', 'sans-serif']
            },
            fontSize: {
                // Biomondo sizes for digital
                'bio-h1': ['42px', { lineHeight: '1.3', fontWeight: '600' }],
                'bio-h2': ['30px', { lineHeight: '1.3', fontWeight: '400' }],
                'bio-body': ['17px', { lineHeight: '1.5', fontWeight: '300' }]
            }
        },
        container: {
            center: true,
        }
    },
}