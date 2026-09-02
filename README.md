# Image converter tool

写真をほかのフォーマットに変換する。JPEG限定で、GPSのEXIFを消去する。元画像は上書きせず、JPEG画像の再圧縮も行わない。

## 動作環境

* Windows

## 機能

### 画像フォーマット変換機能

以下の形式に変換可能

* Jpeg
* Png
* WebP
* ico

画像のメタデータに含まれるGPSデータを削除する。
（機能を追加予定）

### 一括リネーム

EXIFを用いて、画像をリネームする

変換テンプレート

YYYYMMDD_撮影カメラ_連番.Format

(オプションとして、場所を追加可能　例： YYYYMMDD_撮影カメラ_連番_場所.Format )

## License/ライセンス

This project is licensed under the GNU General Public License v3.0 or later.

See the [LICENSE](LICENSE) file for details.

## Fonts

The bundled font is licensed under the SIL Open Font License 1.1.
See `licenses/OFL-1.1.txt` for details.

## ToDo

* 画像変換を別スレットに移行
