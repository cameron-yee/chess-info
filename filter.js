function isUserWhite(username, game) {
  return game.white.username === username
}

function isUserBlack(username, game) {
  return game.black.username === username
}

function filterUserGamesByTimeControlAndPieces(username, pieces, timeClass) {
  return function (game) {
    if (game.time_class !== timeClass) {
      return false
    }
    if (pieces === 'black') {
      return isUserBlack(username, game)
    }
    if (pieces === 'white') {
      return isUserWhite(username, game)
    }

    return false
  }
}

module.exports = {
  isUserWhite,
  isUserBlack,
  filterUserGamesByTimeControlAndPieces
}
