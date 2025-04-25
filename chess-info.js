const { argv } = require('node:process')


async function fetchGamesByYear(username, year) {
  try {
    let allGames = []
    const currentMonth = new Date().getMonth() + 1

    for (let i = 1; i < currentMonth; i++) {
      let month = String(i).padStart(2, '0')

      const resp = await fetch(
        `https://api.chess.com/pub/player/${username}/games/${year}/${month}`,
        {
          method: 'GET'
        }
      )

      const json = await resp.json()
      const games = json.games
      allGames = [...allGames, ...games]
    }

    return allGames
  } catch (err) {
    console.error(err)
  }
}

function getOpening(game) {
  const openingUrlMatches = game.pgn.match(/\[ECOUrl "([^"]+)"/)
  const openingUrl = openingUrlMatches?.[1]
  const openingUrlParts = openingUrl?.split('/')
  const opening = openingUrlParts?.[openingUrlParts?.length - 1]
  return opening
}

function isUserWhite(username, game) {
  return game.white.username === username
}

function isUserBlack(username, game) {
  return game.black.username === username
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

function formatGameData(username) {
  return function mapGame(game) {
    return {
      accuracy: getGameAccuracy(username, game),
      opening: getOpening(game),
      result: getGameResult(username, game)
    }
  }
}

async function getOpeningsByYear(username, year, pieces, timeClass) {
  try {
    const allGames = await fetchGamesByYear(username, year)
    const filteredGames =  allGames.filter(
      filterUserGamesByTimeControlAndPieces(username, pieces, timeClass)
    )

    return filteredGames.map(formatGameData(username))
  } catch (err) {
    console.error(err)
    return []
  }
}

function sortOpeningCounts(openingCounts) {
  return Object.fromEntries(
    Object.entries(openingCounts).sort(([,a],[,b]) => b.count-a.count)
  )
}

function getOpeningCounts(openings) {
  return sortOpeningCounts(openings.reduce((acc, { accuracy, opening, result }) => {
    if (acc[opening]) {
      acc[opening].count++

      if (acc[opening].averageAccuracy && accuracy) {
        acc[opening].averageAccuracy = (acc[opening].averageAccuracy + accuracy) / 2
      } else if (accuracy) {
        acc[opening].averageAccuracy = accuracy
      }

      acc[opening].results[result] = acc[opening].results[result]
        ? acc[opening].results[result] + 1
        : 1

      return acc
    }

    acc[opening] = {}
    acc[opening].count = 1

    if (accuracy) {
      acc[opening].averageAccuracy = accuracy
    }
    acc[opening].results = {}
    acc[opening].results[result] = 1
    return acc
  }, {}))
}

async function main() {
  const [_, __, ...filters] = argv
  if (filters.length !== 4 || filters.includes('-h') || filters.includes('--help')) {
    console.error('node chess-info.js [white|black] [rapid|blitz|bullet] <username> <year>')
    return
  }
  const [pieces, timeClass, username, year] = filters

  const openings = await getOpeningsByYear(username, year, pieces, timeClass)
  const openingCounts = getOpeningCounts(openings)
  console.log(JSON.stringify(openingCounts, null, 2))
}

main()
