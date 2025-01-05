# NYR

## Install
```bash
git clone git@github.com:jonnyroutley/nyr.git

cd nyr

cargo install --path .
```

## Run
```bash
nyr -h

# list targets
nyr targets list

# create target
nyr targets create --name Films --target-value 24

# list progress records
nyr records list

# create progress record
nyr records create --target-id 1 --item-name "Zodiac (2007)"
```
