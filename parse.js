const { isUserBlack, isUserWhite } = require('./filter')

function parsePGN(pgn) {
  let pgnJSON = {
    tags: {},
    moves: ''
  }

  const lines = pgn.trim().split('\n')
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]

    if (/^\[/.test(line)) {
      const matches = line.match(/^\[([^\]]+)\]/)
      const tags = matches?.[1]
      if (tags) {
        const [key, value] = tags.split(' ')
        pgnJSON.tags[key] = value
      }
    }

    pgnJSON.moves = line
  }

  return pgnJSON
}

function getOpening(game) {
  const pgn = parsePGN(game.pgn)
  const openingUrlParts = pgn.tags.ECOUrl?.split('/')
  const opening = openingUrlParts?.[openingUrlParts?.length - 1]
  return opening
}

function getGameResult(username, game) {
  if (isUserWhite(username, game)) {
    return game.white.result
  }
  if (isUserBlack(username, game)) {
    return game.black.result
  }

  return ''
}

function getGameAccuracy(username, game) {
  if (isUserWhite(username, game)) {
    return game.accuracies?.white
  }
  if (isUserBlack(username, game)) {
    return game.accuracies?.black
  }

  return ''
}

module.exports = {
  getOpening,
  getGameResult,
  getGameAccuracy
}

