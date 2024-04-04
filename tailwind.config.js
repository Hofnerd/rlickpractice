const { fontFamily } = require('tailwindcss/defaultTheme');

/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['./templates/*.html'],
    theme: {
        extend: {
            fontFamily: {
                sans: ['Inter var', ...fontFamily.sans],
            },
            colors: {
                'sail': {
                     '50' : '#f2f9fd',
                    '100' : '#e4f1fa',
                    '200' : '#c4e4f4',
                    '300' : '#8eceeb',
                    '400' : '#52b4de',
                    '500' : '#2c9ccb',
                    '600' : '#1d7cac',
                    '700' : '#19648b',
                    '800' : '#185574',
                    '900' : '#194761',
                    '950' : '#112d40', 
                },
            },
        },
    },
    plugins: [require('@tailwindcss/forms')],
}

