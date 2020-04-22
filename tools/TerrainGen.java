public class TerrainGen {
    public static void main(String[] args) {
        
    }

    private static StringBuilder display(int dist, int xView, int yView) {
        StringBuilder s = new StringBuilder();
        for(int y = max(0,yView - dist); y < min(World.MAP_SIZE, yView + dist + 1); y++) {
            s.append("|");
            for(int x = max(0,xView - dist); x < min(World.MAP_SIZE, xView + dist + 1); x++) {
                boolean hasPlayer = false;
                for(Player player : accept.players()) {
                    if(player.x() == x && player.y() == y) {
                        s.append("\033[0m" + player.playerRep);
                        hasPlayer = true;
                        break;
                    }
                }
                if(!hasPlayer) {
                    Block b = world.getBlock(x, y);
                    int asciiColor = (Integer)b.getStats().get("display");
                    boolean hideMob = b.getStats().hasVariable("hide-mobs") && (Boolean)b.getStats().get("hide-mobs");
                    if(asciiColor == -1) {
                        if(!hideMob && world.hasMob(x, y)) {
                            s.append("??");
                        } else {
                            s.append("  ");
                        }
                    } else {
                        if(!hideMob && world.hasMob(x, y)) {
                            s.append("\033[48;5;" + asciiColor + "m??");
                        } else {
                            s.append("\033[48;5;" + asciiColor + "m  ");
                        }
                    }
                }
            }
            s.append("\033[0m|\n");
        }
        return s;
    }
}