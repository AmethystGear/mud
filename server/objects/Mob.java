import java.io.*;
import java.util.*;

public class Mob {
    private Stats baseStats;
    private Stats stats;
    private StringBuilder img = new StringBuilder("");

    public Mob(int mobType, String mobFile) {
        baseStats = new Stats();
        stats = new Stats();
        try {
            int mob = 0;
            boolean gettingImg = false;
            Scanner fr = new Scanner(new File(mobFile));
            while (fr.hasNextLine()) {
                String data = fr.nextLine();
                if(data.equals("")) { //ignore empty lines
                    continue;
                }
                if(data.strip().equals("/begin/")) {
                    mob++;
                    if(mob == mobType) {
                        baseStats = new Stats(fr);
                        stats = baseStats.clone();
                    }
                }
                else if(mob == mobType) {                    
                    if(data.strip().equals("img:")) {
                        gettingImg = true;
                    } else if(gettingImg) {
                        img.append(data);
                        img.append('\n');
                    }
                }
            }
            fr.close();
        } catch (FileNotFoundException e) {
            e.printStackTrace();
        }
        stats = baseStats.clone();
    }

    public ReadOnlyStats getBaseStats() {
        return new ReadOnlyStats(stats);
    }

    public ReadOnlyStats getStats() {
        return new ReadOnlyStats(stats);
    }

    public void changeStat(String stat, int amount) {
        int currentAmount = (Integer)stats.get(stat);
        stats.set(stat, currentAmount + amount);
    }

    public boolean isDead() {
        return (Integer)stats.get("health") <= 0;
    }

    public String[] getDrops() {
        String[] drops;
        try {
            drops = (String[])baseStats.get("drops");
        } catch (IllegalArgumentException e) {
            return new String[0];
        }
        int min;
        int max;
        try {
            min = (Integer)getBaseStats().get("drop-min");
        } catch (IllegalArgumentException e) {
            min = 1;
        }
        try {
            max = (Integer)getBaseStats().get("drop-max");
        } catch (IllegalArgumentException e) {
            max = min;
        }
        int numDrops = RandUtils.rand(min, max);
        String[] playerDrops = new String[numDrops];
        for(int i = 0; i < playerDrops.length; i++) {
            playerDrops[i] = RandUtils.getRandom(drops);
        }
        return playerDrops;
    }

    public String getQuote(String quoteType) {
        try {
            return RandUtils.getRandom((String[])stats.get(quoteType));
        } catch(IllegalArgumentException e) {
            return "";
        }
    }

    public String getImg() {
        return img.toString();
    }
}