package server.actions;

import java.util.List;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.utils.ScannerUtils;

public class Mark implements Action {
    private static HashMap<Integer, ArrayList<Marker>> markers = new HashMap<>();

    String descr;

    @Override
    public boolean matchCommand(String command) {
        if(command.equals("markers")) {
            return true;
        }
        return command.startsWith("mark");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        if(command.equals("markers")) {
            return true;
        } else {
            Scanner scan = new Scanner(command);
            scan.next();
            if(!scan.hasNext()) {
                scan.close();
                error.append("you need to type a description of your marker!");
                return false;
            }
            descr = ScannerUtils.getRemainingInputAsString(scan);
            return true;
        }
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        if(!markers.containsKey(player.ID())) {
            markers.put(player.ID(), new ArrayList<>());
        }

        if(descr == null) {
            StringBuilder out = new StringBuilder();
            for(Marker m : markers.get(player.ID())) {
                out.append(m + "\n");
            }
            return out;
        } else {
            markers.get(player.ID()).add(new Marker(player.x(), player.y(), descr));
            return new StringBuilder("successfully created marker " + descr + " at " + player.x() + ", " + player.y());
        }
    }

    @Override
    public String description() {
        return "lets you mark places you've been to, and view all the markers you've placed.\n" +
               "usage: markers | mark <description>\n" +
               "example: markers (this would display all the markers you've placed)\n" +
               "example: mark Honour Mage (this would add a marker to your list named Honour Mage. The marker would be placed at your current position.)\n";
    }

    private static class Marker {
        private int x;
        private int y;
        private String description;
        
        public Marker(int x, int y, String description) {
            this.x = x;
            this.y = x;
            this.description = description;
        }

        public String toString() {
            return "(" + x + ", " + y + ") - " + description;
        }
    }
}