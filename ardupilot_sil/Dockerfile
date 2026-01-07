FROM ubuntu:22.04

ARG COPTER_TAG=Copter-4.5.7
# Trick to get apt-get to not prompt for timezone in tzdata
ENV DEBIAN_FRONTEND=noninteractive
# install git 
RUN apt-get update && apt-get install -y git; git config --global url."https://github.com/".insteadOf git://github.com/
RUN apt-get install -y sudo lsb-release tzdata python3 python3-pip

RUN useradd -m ardupilot && \
    echo "ardupilot ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ardupilot && \
    usermod -a -G sudo,dialout ardupilot
# Switch to non-root user for installation
USER ardupilot

WORKDIR /ardupilot
# Now grab ArduPilot from GitHub
RUN git clone --depth 1 --branch ${COPTER_TAG} https://github.com/ArduPilot/ardupilot.git /ardupilot

# Now start build instructions from http://ardupilot.org/dev/docs/setting-up-sitl-on-linux.html
RUN git -C /ardupilot submodule update --init --recursive --depth 10

# Need sudo and lsb-release for the installation prerequisites

# Install Python packages separately to handle empy version issue
RUN pip3 install --user future lxml pymavlink pyserial MAVProxy pexpect geocoder ptyprocess dronecan flake8 junitparser pygame intelhex && \
    pip3 install --user empy==3.3.4 --no-deps

ENV USER=ardupilot

RUN Tools/environment_install/install-prereqs-ubuntu.sh -y

# Continue build instructions from https://github.com/ArduPilot/ardupilot/blob/master/sBUILD.md
RUN ./waf distclean
RUN ./waf configure --board sitl
RUN ./waf copter


# Variables for simulator
ENV COUNT 1
#34°12'25.2"N 117°24'04.6"W
ENV LAT 34.20699
ENV LON -117.40128
ENV ALT 14
ENV DIR 270
ENV MODEL +
ENV SPEEDUP 1
ENV VEHICLE ArduCopter

# Finally the command
ENTRYPOINT /ardupilot/Tools/autotest/sim_vehicle.py \
    --vehicle ${VEHICLE} \
    --count ${COUNT} \
    --custom-location=${LAT},${LON},${ALT},${DIR} \
    -w \
    --frame ${MODEL} \
    --no-rebuild \
    --no-mavproxy \
    --speedup ${SPEEDUP} \
    --auto-offset-line 90,4 \ 