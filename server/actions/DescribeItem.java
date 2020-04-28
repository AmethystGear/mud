package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.objects.Item;

public class DescribeItem implements Action {
    Item item;

    @Override
    public boolean matchCommand(String command) {
        return command.startsWith("descr");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        String itemStr;
        if(!scan.hasNext()) {
            scan.close();
            error.append("you need to type the name of the item!");
            return false;
        } else {
            itemStr = scan.next();
        }
        scan.close();
        try {
            item = world.items.get(itemStr);
            if(!item.getStats().hasVariable("description")) {
                throw new RuntimeException();
            }
        } catch(Exception e) {
            error.append("either that item doesn't exist, or it doesn't have a description.");
            return false;
        }
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        String description = (String)item.getStats().get("description");
        return new StringBuilder(description + "\n");
    }

    @Override
    public String description() {
        return "describes an item.\n" + 
               "usage: descr <item name>\n";
    }

}