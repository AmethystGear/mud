package server.objects.instantiables;

import server.utils.RandUtils;

import java.util.List;

import server.main.World;
import server.objects.Entity;
import server.objects.Instantiable;
import server.objects.Player;
import server.objects.Spawnable;
import server.objects.Stats;

public class Mob implements Instantiable<Mob>, Spawnable {
    private Stats stats;
    private Stats baseStats;
    private int accumSpeed;

    public Mob() {
    }

    public Mob(Stats.ReadOnlyStats stats) {
        this.stats = stats.clone();
        this.baseStats = stats.clone();
        this.accumSpeed = (Integer) stats.get("speed");
    }

    @Override
    public Mob create(Entity e) {
        return new Mob(e.getStats());
    }

    public Stats.ReadOnlyStats getStats() {
        return new Stats.ReadOnlyStats(stats);
    }

    public Stats.ReadOnlyStats getBaseStats() {
        return new Stats.ReadOnlyStats(baseStats);
    }

    public boolean isDead() {
        return (Integer) stats.get("health") <= 0;
    }

    public String[] getDrops() {
        if (!stats.hasVariable("drops")) {
            return new String[0];
        }
        String[] drops = (String[]) baseStats.get("drops");
        int min = stats.hasVariable("drop min") ? (Integer) getBaseStats().get("drop min") : 1;
        int max = stats.hasVariable("drop min") ? (Integer) getBaseStats().get("drop min") : 1;
        int numDrops = RandUtils.rand(min, max);
        String[] playerDrops = new String[numDrops];
        for (int i = 0; i < playerDrops.length; i++) {
            playerDrops[i] = RandUtils.getRandom(drops);
        }
        return playerDrops;
    }

    public String getQuote(String quoteType) {
        if (!stats.hasVariable(quoteType)) {
            return "";
        }
        quoteType = quoteType.replace('-', ' ');
        return RandUtils.getRandom((String[]) stats.get(quoteType));
    }

    public String getImg() {
        return baseStats.hasVariable("img") ? ((StringBuilder) baseStats.get("img")).toString() : "\n";
    }

    public void changeStat(String stat, int amount) {
        int currentAmount = (Integer) stats.get(stat);
        stats.set(stat, currentAmount + amount);
    }

    public StringBuilder attack(Player player, World world) {
        StringBuilder out = new StringBuilder("");
        int playerSpeed = (Integer) player.getStats().get("speed");
        int mobSpeed = (Integer) stats.get("speed");

        accumSpeed += mobSpeed;
        if (accumSpeed < playerSpeed) {
            out.append(stats.get("name") + " is too slow. You take another turn.\n");
        } else {
            accumSpeed = 0;
            for (int speed = 0; speed < mobSpeed; speed += playerSpeed) {
                out.append(getBaseStats().get("name") + ": " + getQuote("attack") + "\n");
                out.append(getBaseStats().get("name") + " attacked you and dealt " + getBaseStats().get("dmg")
                        + " damage.\n");
                player.changeStat("health", -(Integer) getStats().get("dmg"));
                if (player.isDead()) {
                    out.append(getBaseStats().get("name") + ": " + getQuote("mob victory") + "\n");
                    out.append("You were killed by " + getBaseStats().get("name") + "\n");
                    player.respawn(world);
                    out.append("Respawning at " + player.x() + ", " + player.y() + "\n");
                }
                if (player.isDead()) {
                    return out;
                }
                if (speed + playerSpeed <= mobSpeed) {
                    out.append(stats.get("name") + " is faster than you, and takes another turn.\n");
                }
            }
        }
        return out;
    }

    public static class ReadOnlyMob {
        private Mob m;

        public ReadOnlyMob(Mob m) {
            this.m = m;
        }

        public Stats.ReadOnlyStats getStats() {
            return m.getStats();
        }

        public Stats.ReadOnlyStats getBaseStats() {
            return m.getBaseStats();
        }

        public boolean isDead() {
            return m.isDead();
        }

        public String[] getDrops() {
            return m.getDrops();
        }

        public String getQuote(String quoteType) {
            return m.getQuote(quoteType);
        }

        public String getImg() {
            return m.getImg();
        }
    }

    @Override
    public StringBuilder interact(Player player, List<Player> players, World world) {
        StringBuilder s = new StringBuilder();
        s.append("You encountered: ");
        s.append(getBaseStats().get("name") + "\n");
        player.setMob(this);
        s.append(getImg() + "\n");
        s.append(getQuote("entrance") + "\n");
        int playerSpeed = (Integer) player.getStats().get("speed");
        int mobSpeed = (Integer) getStats().get("speed");
        if (playerSpeed < mobSpeed) {
            s.append(attack(player, world));
            s.append("\n");
        }
        return s;
    }
}
