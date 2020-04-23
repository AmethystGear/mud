package server.actions;

import java.util.List;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

import server.utils.DisplayUtils;

public class Display implements Action {

    @Override
    public boolean matchCommand(String command) {
        return command.equals("disp");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world, StringBuilder error) {
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        int viewDist = (Integer)player.getBaseStats().get("view");
        return DisplayUtils.display(viewDist, player.x(), player.y(), players, world, true);
    }
}