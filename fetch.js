async function fetchGamesByUsernameAndYear(username, year) {
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
    return []
  }
}

module.exports = {
  fetchGamesByUsernameAndYear
}
