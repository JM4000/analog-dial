int dataArray[] = { -1, -1, -1, -1 };
int count = 0;
int cpu = 5, mem = 6, gpu = 9, net = 10; 

void setup() {
  Serial.begin(9600);  // opens serial port, sets data rate to 9600 bps
  Serial.flush();

  pinMode(cpu, OUTPUT);
  pinMode(gpu, OUTPUT);
  pinMode(mem, OUTPUT);
  pinMode(net, OUTPUT);
}

void loop() {
  dataInput();
  if (dataArray[3] != -1) {
    delay(1000);
    analogWrite(cpu, dataArray[0]);
    analogWrite(mem, dataArray[1]);
    analogWrite(gpu, dataArray[2]);
    analogWrite(net, dataArray[3]);
    for (int i = 0; i < 4; i++) {
      Serial.write(dataArray[i]);
    }
    emptyData();
  }
}

void addData(int data) {
  if (data == 0) {
    //RESET
    analogWrite(11, 0);
    emptyData();
  } else {
    dataArray[count] = data;
    count++;
  }
}

void emptyData() {
  for (int i = 0; i < 4; i++) {
    dataArray[i] = -1;
  }
  count = 0;
}

void dataInput() {
  while (Serial.available() == 0) {}  //wait for data available
  int incomingData = Serial.read();   // empty the serial buffer by reading everything from it
  Serial.flush();
  addData(incomingData);
  delay(500);
}