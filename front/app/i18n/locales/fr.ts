import type { Translations } from './en'

const fr: Translations = {
  common: {
    loading: 'Chargement…',
    loadFailed: 'Chargement impossible',
  },
  language: {
    switch: 'Switch to English',
  },
  login: {
    description: 'Connectez-vous pour retrouver vos parties.',
    login: 'Identifiant',
    submit: 'Se connecter',
    failed: 'Connexion impossible',
  },
  home: {
    title: 'Mes parties',
    signOut: 'Se déconnecter',
    newGame: 'Nouvelle partie',
    createFailed: 'Création impossible',
    empty: 'Aucune partie pour l’instant. Lancez-en une !',
  },
  status: {
    in_progress: 'En cours',
    solved: 'Résolue',
    surrendered: 'Abandonnée',
  },
  game: {
    title: 'Partie',
    back: 'Retour aux parties',
    surrender: 'Se rendre',
    surrenderConfirm: 'Abandonner cette partie ? Les pièces cachées seront révélées.',
    surrenderFailed: 'Abandon impossible',
    shotFailed: 'Tir impossible',
    solved: 'Partie résolue : voici les pièces cachées. Le tir est désactivé.',
    surrendered: 'Partie abandonnée : voici les pièces cachées. Le tir est désactivé.',
    grid: 'Grille',
    gridRevealed: 'Grille et pièces cachées',
    removePiece: 'Retirer la pièce « {{shape}} »',
    entry: 'Entrée {{entry}}',
    entryShot: 'Entrée {{entry}}, rayon {{label}} : {{entry}} → {{other}}, {{colour}}',
  },
  shots: {
    title: 'Tirs',
    empty: 'Cliquez sur une entrée autour de la grille pour tirer un rayon.',
  },
  draft: {
    title: 'Brouillon',
    propose: 'Proposer',
    proposeFailed: 'Proposition impossible',
    placed: '{{count}} / {{total}} pièces placées',
    incorrect: 'Incorrect',
    pieceToPlace: 'Pièce à placer',
    placedSuffix: ' (placée)',
    rotate: 'Tourner',
    rotateHint: 'Tourner d’un quart de tour (R)',
    colour: 'Couleur',
    cell: 'Case {{x}}, {{y}}',
    cellRemove: 'Case {{x}}, {{y}} : retirer la pièce « {{shape}} »',
    cellPlace: 'Case {{x}}, {{y}} : placer la pièce « {{shape}} »',
  },
  shapes: {
    square: 'carré',
    bar: 'barre',
    spike: 'pointe',
    corner: 'coin',
    diamond: 'losange',
  },
  colours: {
    none: 'Aucune',
    white: 'Blanc',
    red: 'Rouge',
    blue: 'Bleu',
    yellow: 'Jaune',
    violet: 'Violet',
    orange: 'Orange',
    green: 'Vert',
    black: 'Noir',
  },
}

export default fr
