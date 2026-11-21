export const THEME_STORAGE_KEY = 'scout-beam-theme'

/** Runs in <head> before paint, so a light theme never flashes dark first. */
export const THEME_SCRIPT = `try{if(localStorage.getItem('${THEME_STORAGE_KEY}')==='light')document.documentElement.classList.remove('dark')}catch{}`
