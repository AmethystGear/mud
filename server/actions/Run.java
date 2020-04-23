package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class Run implements Action {
    @Override
    public boolean matchCommand(String command) {
        return command.equals("run");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        if (player.getMob() == null) {
            error.append("you can't run because you aren't currently fighting a mob!");
        }
        return player.getMob() != null;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder out = new StringBuilder("");
        out.append(player.getMob().getBaseStats().get("name") + ": " + player.getMob().getQuote("player-run") + "\n");
        out.append("You ran away from " + player.getMob().getBaseStats().get("name") + ".\n");
        player.setMob(null);
        return out;
    }
}