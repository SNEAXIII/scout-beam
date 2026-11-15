import type { Translations } from './en'

const fr: Translations = {
  common: {
    loading: 'Chargement…',
    loadFailed: 'Chargement impossible',
    cancel: 'Annuler',
    confirm: 'Confirmer',
  },
  language: {
    switch: 'Switch to English',
  },
  keyboard: {
    switch: 'Changer la disposition du clavier',
    azerty: 'AZERTY',
    qwerty: 'QWERTY',
  },
  hotkeys: {
    legend: 'Raccourcis clavier',
    shape: 'forme',
    colour: 'couleur',
    rotate: 'tourner',
    remove: 'retirer',
    rightClick: 'Clic droit',
    space: 'Espace',
  },
  login: {
    description: 'Connectez-vous pour retrouver vos parties.',
    login: 'Identifiant',
    password: 'Mot de passe',
    submit: 'Se connecter',
    failed: 'Connexion impossible',
    or: 'ou',
    continueWith: 'Continuer avec {{provider}}',
    noAccount: 'Pas encore de compte ?',
    createAccount: 'En créer un',
    devTitle: 'Connexion développeur',
    devDescription:
      'Uniquement en développement : le compte est créé à la première connexion, sans mot de passe.',
    devSubmit: 'Se connecter sans mot de passe',
  },
  providers: {
    discord: 'Discord',
    google: 'Google',
  },
  register: {
    title: 'Créer un compte',
    description:
      'Un identifiant de 3 à 30 lettres ou chiffres, et un mot de passe de 8 à 128 caractères.',
    confirm: 'Confirmez le mot de passe',
    submit: 'Créer le compte',
    mismatch: 'Les deux mots de passe ne correspondent pas',
    failed: 'Impossible de créer le compte',
    haveAccount: 'Déjà un compte ?',
    signIn: 'Se connecter',
  },
  callback: {
    pending: 'Connexion en cours…',
    invalid: 'Ce lien de connexion est invalide ou a expiré. Recommencez.',
    failed: 'Connexion impossible avec ce fournisseur',
    back: 'Retour à la connexion',
  },
  play: {
    title: 'Mes parties',
    signOut: 'Se déconnecter',
    newGame: 'Nouvelle partie',
    createFailed: 'Création impossible',
    empty: 'Aucune partie pour l’instant. Lancez-en une !',
  },
  landing: {
    title: 'Scout Beam',
    tagline:
      'Un jeu de déduction : tirez des rayons dans la grille et retrouvez où sont cachées les pièces.',
    play: 'Jouer',
    signIn: 'Se connecter',
    rulesTitle: 'Comment jouer',
    piecesLabel: 'Les cinq pièces',
    beamDemo:
      'Un rayon entre par le haut, tourne de 90° sur un côté oblique rouge, fait demi-tour sur un côté droit bleu et ressort par sa propre entrée, violet.',
    colourMix:
      'Rouge et bleu donnent violet, rouge et jaune orange, bleu et jaune vert, les trois noir.',
    rules: {
      grid: 'Cinq pièces sont cachées sur une grille de 10 × 8. Tout autour, 36 entrées numérotées : c’est par là que les rayons entrent et sortent.',
      pieces:
        'Les pièces sont le carré, la barre, la pointe, le coin et le losange. Chacune a une couleur : le rouge, le bleu et le jaune sont toujours présents.',
      shot: 'Un tir envoie un rayon depuis une entrée. Il a pour réponse l’entrée par laquelle le rayon ressort et la couleur prise en chemin.',
      sides:
        'Un côté droit renvoie le rayon d’où il vient (180°) ; un côté oblique le fait tourner de 90°.',
      colours:
        'La couleur du rayon mélange celles des pièces touchées : rouge et bleu donnent violet, rouge et jaune orange, bleu et jaune vert, les trois noir. Des pièces blanches seules donnent blanc, rien du tout ne donne aucune couleur.',
      draft:
        'Placez un brouillon sur la grille, là où vous pensez que sont les pièces, puis soumettez-le comme proposition : elle est juste si elle donne le même résultat que la disposition cachée sur les 36 entrées.',
      surrender: 'Bloqué ? Abandonner révèle la disposition cachée et termine la partie.',
    },
    inspiredBy: 'Inspiré d’',
    game: 'Orapa Mine',
    credit: ', de Junghee Choi et Wanjin Gill (Playte, 2024 ; édité en France par Miraludo)',
  },
  status: {
    in_progress: 'En cours',
    solved: 'Résolue',
    surrendered: 'Abandonnée',
  },
  game: {
    title: 'Partie',
    back: 'Retour aux parties',
    surrender: 'Abandonner',
    surrenderTitle: 'Abandonner cette partie ?',
    surrenderDescription: 'Les pièces cachées seront révélées et le tir sera désactivé.',
    surrenderFailed: 'Abandon impossible',
    shotFailed: 'Tir impossible',
    offline: 'Pas connecté à la partie : réessayez dans un instant',
    reconnecting: 'Connexion perdue, reconnexion…',
    solved: 'Partie résolue : voici les pièces cachées. Le tir est désactivé.',
    surrendered: 'Partie abandonnée : voici les pièces cachées. Le tir est désactivé.',
    grid: 'Grille',
    gridRevealed: 'Grille et pièces cachées',
    removePiece: 'Clic droit ou Espace pour retirer la pièce « {{shape}} »',
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
    shapeHint: '{{shape}} ({{previous}} / {{next}})',
    colourHint: '{{colour}} ({{previous}} / {{next}})',
    cell: 'Case {{x}}, {{y}}',
    cellPiece: 'Case {{x}}, {{y}} : pièce « {{shape}} »',
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
