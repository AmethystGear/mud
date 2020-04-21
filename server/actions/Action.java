package server.actions;

import server.World;
import server.objects.Player;

import java.util.List;

public interface Action {
    // quick check to see if the command is referring to this action.
    // do the minimum work required to determine whether the player wants to run this action specifically or not.
    public boolean matchCommand(String command);

    // actually parse and store the command parameters in your class.
    // if the player is in some kind of state where they shouldn't be able to run this command, then return false.
    // if you have some kind of error/formatting message to display, use the error parameter and append your error to that.
    public boolean parseCommand(String command, Player.ReadOnlyPlayer player, List<Player.ReadOnlyPlayer> players, World world, StringBuilder error);

    // run the action with the stored parameters. You can assume the parameters are legal in this function because
    // you should have verified them during parseCommand.
    public StringBuilder run(Player player, List<Player> players, World world);
}