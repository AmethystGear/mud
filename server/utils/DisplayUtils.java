package server.utils;

import java.util.List;
import java.util.ArrayList;

import server.main.World;
import server.objects.Block;
import server.objects.Player;

public class DisplayUtils {

    public static StringBuilder map(int chunkSize, List<Player> players, World world) {
        StringBuilder s = new StringBuilder();
        for (int y = 0; y < World.MAP_SIZE; y += chunkSize) {
            s.append("|");
            for (int x = 0; x < World.MAP_SIZE; x += chunkSize) {
                boolean hasPlayer = false;
                for (Player player : players) {
                    if (player.x() >= x && player.x() < x + chunkSize && player.y() > y
                            && player.y() <= y + chunkSize) {
                        s.append("\033[0m" + player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if (!hasPlayer) {
                    int majorityBlock = getMajorityBlockInChunk(x, y, chunkSize, world);
                    int asciiColor = (Integer) world.blocks.get(majorityBlock).getStats().get("display");
                    if (asciiColor == -1) {
                        s.append("  ");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m  ");
                    }
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }

    private static int getMajorityBlockInChunk(int xOrigin, int yOrigin, int chunkSize, World world) {
        ArrayList<Integer> blockList = new ArrayList<Integer>();
        for (int x = xOrigin; x < xOrigin + chunkSize; x++) {
            for (int y = yOrigin; y < yOrigin + chunkSize; y++) {
                Block b = world.getBlock(x, y);
                while (blockList.size() <= b.getID()) {
                    blockList.add(0);
                }
                int mapWeight;
                if (b.getStats().hasVariable("map weight")) {
                    mapWeight = (Integer) b.getStats().get("map weight");
                } else {
                    mapWeight = 1;
                }
                blockList.set(b.getID(), blockList.get(b.getID()) + mapWeight);
            }
        }
        int maxIndex = 0;
        for (int i = 1; i < blockList.size(); i++) {
            if (blockList.get(i) > blockList.get(maxIndex)) {
                maxIndex = i;
            }
        }
        return maxIndex;
    }

    public static StringBuilder display(int dist, int xView, int yView, List<Player> players, World world,
            boolean showMobs) {
        StringBuilder s = new StringBuilder();
        for (int y = MathUtils.max(0, yView - dist); y < MathUtils.min(World.MAP_SIZE, yView + dist + 1); y++) {
            s.append("|");
            for (int x = MathUtils.max(0, xView - dist); x < MathUtils.min(World.MAP_SIZE, xView + dist + 1); x++) {
                int leastDist = Integer.MAX_VALUE;
                boolean hasPlayer = false;
                for (Player player : players) {
                    int manhattanDist = MathUtils.manhattan(x, y, player.x(), player.y());
                    if (manhattanDist < leastDist) {
                        leastDist = manhattanDist;
                    }
                    if (player.x() == x && player.y() == y) {
                        s.append("\033[0m" + player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if (!hasPlayer) {
                    Block b = world.getBlock(x, y);
                    int asciiColor = (Integer) b.getStats().get("display");
                    if (world.hasMob(x, y) && showMobs) {
                        s.append("\033[48;5;" + asciiColor + "m??");
                    } else {
                        s.append("\033[48;5;" + asciiColor + "m  ");
                    }
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }
}