# Send ^C to dbobs screen
screen -S dbobs -X stuff $'\003'
./dbobs-screen.sh
