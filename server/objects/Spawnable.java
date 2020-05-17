package server.objects;

import java.util.List;
import server.main.World;

public interface Spawnable {
    // This method will be called when the player steps onto this object
    public StringBuilder interact(Player player, List<Player> players, World world);
}
