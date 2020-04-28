package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.utils.ScannerUtils;
import server.objects.Item;

public class Eat implements Action {
    // parameters
    private int numToEat;
    private Item item;

    @Override
    public boolean matchCommand(String command) {
        Scanner scan = new Scanner(command);
        boolean result = scan.hasNext() && scan.next().equals("eat");
        scan.close();
        return result;
    }

    @Override
    public boolean parseCommand(String command, Player.ReadOnlyPlayer player, List<Player.ReadOnlyPlayer> players,
            World world, StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if (!scan.hasNextInt()) {
            scan.close();
            error.append("you need to specify the amount you are going to eat!");
            return false;
        }
        numToEat = scan.nextInt();
        String itemName = ScannerUtils.getRemainingInputAsString(scan).replace('-', ' ');
        if (!player.getInventory().hasVariable(itemName)) {
            error.append("you don't have that item!");
            return false;
        }

        try {
            item = world.items.get(itemName);
            if (!item.getStats().hasProperty("edible")) {
                error.append("you can't eat that item!");
                return false;
            }
        } catch (IllegalArgumentException e) {
            error.append("you can't eat that item!");
            return false;
        }

        if (numToEat < 0) {
            error.append("you can't eat a negative number of items!");
            return false;
        }
        if (numToEat > (Integer) player.getInventory().get(itemName)) {
            error.append("you only have " + ((Integer) player.getInventory().get(itemName)) + " of that item!");
            return false;
        }
        scan.close();
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        player.removeFromInventory((String) item.getStats().get("name"), numToEat);
        player.changeStat("health", numToEat * (Integer) item.getStats().get("health gain"));
        return new StringBuilder("you got " + (numToEat * (Integer) item.getStats().get("health gain")) + " health.");
    }

    @Override
    public String description() {
        return "eat items in your inventory.\n" +
               "usage: eat <number of items> <name of item>\n";
    }
}