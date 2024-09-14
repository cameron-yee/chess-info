const { argv } = require('node:process')

async function fetchGamesByYear(username, year) {
  try {
    let allGames = []
    const currentMonth = new Date().getMonth() + 1

    for (let i = 1; i < currentMonth; i++) {
      let month = i

      if (i < 10) {
        month = `0${i}`
      }

      const resp = await fetch(`https://api.chess.com/pub/player/${username}/games/${year}/${month}`, {
        method: 'GET'
      })

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
  const openingUrl = game.eco
  const openingUrlParts = openingUrl.split('/')
  const opening = openingUrlParts[openingUrlParts.length - 1]
  return opening
}

function getGameResult(username, game) {
  return game.white.username === username
    ? game.white.result
    : game.black.username === username
      ? game.black.result
      : ''
}

async function getOpeningsByYear(username, year, pieces, timeClass) {
  try {
    const allGames = await fetchGamesByYear(username, year)
    const filteredGames =  allGames.filter((game) => {
        if (game.time_class !== timeClass) {
          return false
        }
        if (pieces === 'black') {
          return game.black.username === username
        }
        if (pieces === 'white') {
          return game.white.username === username
        }

        return false
      })

    const openings = filteredGames.map((game) => {
      const opening = getOpening(game)
      const result = getGameResult(username, game)

      return {
        opening,
        result
      }
    })

    return openings
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
  return sortOpeningCounts(openings.reduce((acc, { opening, result }) => {
    if (acc[opening]) {
      acc[opening].count++
      acc[opening][result] = acc[opening][result] ? acc[opening][result] + 1 : 1
      return acc
    }

    acc[opening] = {}
    acc[opening].count = 1
    acc[opening][result] = 1
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
  console.log(openingCounts)
}

main()
