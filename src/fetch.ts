import { IGame, IChessAPIGamesResponse } from './types'

export async function fetchGamesByUsernameAndYear(username: string, year: string): Promise<IGame[]> {
  try {
    let allGames: IGame[] = []
    const currentMonth = new Date().getMonth() + 1

    for (let i = 1; i < currentMonth; i++) {
      let month = String(i).padStart(2, '0')

      const resp: Response = await fetch(
        `https://api.chess.com/pub/player/${username}/games/${year}/${month}`,
        {
          method: 'GET'
        }
      )

      const json = await resp.json() as IChessAPIGamesResponse
      const games = json.games
      allGames = [...allGames, ...games]
    }

    return allGames
  } catch (err) {
    console.error(err)
    return []
  }
}

