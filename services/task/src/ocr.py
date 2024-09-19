import cv2
from paddleocr import PaddleOCR, PPStructure
from pathlib import Path
from bs4 import BeautifulSoup

from src.models.ocr_model import OCRResult, OCRResponse, TableOCRResponse, BoundingBox


def ppocr_raw(ocr: PaddleOCR, image_path: Path) -> list:
    return ocr.ocr(str(image_path))


def ppocr(ocr: PaddleOCR, image_path: Path) -> OCRResponse:
    raw_results = ocr.ocr(str(image_path))
    ocr_results = [
        OCRResult(
            bbox=BoundingBox(
                top_left=result[0][0],
                top_right=result[0][1],
                bottom_right=result[0][2],
                bottom_left=result[0][3]
            ),
            text=result[1][0],
            confidence=result[1][1]
        )
        for result in raw_results[0]
    ]
    return OCRResponse(results=ocr_results)


def ppstructure_table_raw(table_engine: PPStructure, image_path: Path) -> list:
    img = cv2.imread(str(image_path))
    result = table_engine(img)
    for line in result:
        line.pop('img')
    return result


def ppstructure_table(table_engine: PPStructure, image_path: Path) -> TableOCRResponse:
    img = cv2.imread(str(image_path))
    result = table_engine(img)

    table_result = result[0] if result else None

    if not table_result:
        return TableOCRResponse(results=[], html="")

    cell_bbox_raw = table_result['res'].get('cell_bbox', [])
    html = table_result['res'].get('html', "")

    # Parse the HTML
    soup = BeautifulSoup(html, 'html.parser')
    cells = soup.find_all(['td', 'th'])

    cell_bbox = []
    for bbox, cell in zip(cell_bbox_raw, cells):
        cell_bbox.append(
            OCRResult(
                bbox=BoundingBox(
                    top_left=[bbox[0], bbox[1]],
                    top_right=[bbox[2], bbox[3]],
                    bottom_right=[bbox[4], bbox[5]],
                    bottom_left=[bbox[6], bbox[7]],
                ),
                text=cell.get_text(strip=True)
            )
        )

    response = TableOCRResponse(results=cell_bbox, html=html)
    return response
