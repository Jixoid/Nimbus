## QCL

Modern, Skia tabanlı bir kullanıcı arayüzü (UI) motoru ve iskeleti. 
QAOS çatısı altında geliştirilen QCL, arayüz programlamayı basitleştirmek ve modern prensiplerini benimsemek için temiz ve katmanlı bir mimariye odaklanır.

Diğer diller: [en](README.md)


## Mimari

Proje, birbirinden yüksek oranda ayrıştırılmış modüller halinde yapılandırılmıştır:

1. **QCL Çekirdek (Core):** Temel arayüz bileşenleri (Kontroller, Görünümler, Pencereler) ve sanal dosya sisteminin (VFS) yönetimi.
2. **QSTD (Standart Kütüphane):** Kullanıma hazır, standartlaştırılmış UI bileşenleri paketi.
3. **Platform:** İşletim sistemine özgü entegrasyonlar ve düşük seviyeli arka uç (backend) bağlantıları.


## Görseller

![QCL Arayüz Testi](/images/test.png)
*Arayüz testi.*


## Lisans

Bu proje GNU Genel Kamu Lisansı sürüm 3 (GPL3) altında lisanslanmıştır.

Telif Hakkı (c) 2025-2026 Kadir Aydın.


## QAOS

Açık kaynak topluluğu tarafından ❤️ ile geliştirilmiştir.
